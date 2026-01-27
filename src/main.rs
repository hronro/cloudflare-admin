mod cloudflare;
mod storage;
mod ui;

use cloudflare::{
    CloudflareClient, CreateDnsRecord, DnsRecord, DnsRecordType, UpdateDnsRecord, Zone,
};
use gpui::prelude::*;
use gpui::{
    Application, Bounds, Context, Entity, IntoElement, Render, SharedString, TitlebarOptions,
    Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::{
    ActiveTheme, Root, VirtualListScrollHandle, WindowExt,
    input::InputState,
    notification::Notification,
    select::{SelectEvent, SelectItem, SelectState},
    theme::{Theme, ThemeMode},
};

// Application pages
#[derive(Clone, PartialEq)]
pub enum Page {
    TokenSetup,
    Dashboard,
    Settings,
}

// Appearance mode for theme switching
#[derive(Clone, Copy, PartialEq, Default)]
pub enum AppearanceMode {
    Light,
    Dark,
    #[default]
    Auto,
}

impl AppearanceMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppearanceMode::Light => "light",
            AppearanceMode::Dark => "dark",
            AppearanceMode::Auto => "auto",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "light" => AppearanceMode::Light,
            "dark" => AppearanceMode::Dark,
            _ => AppearanceMode::Auto,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppearanceMode::Light => "Light",
            AppearanceMode::Dark => "Dark",
            AppearanceMode::Auto => "Auto (System)",
        }
    }
}

// Wrapper for Zone to implement SelectItem
#[derive(Clone)]
pub struct ZoneItem {
    pub zone: Zone,
}

impl SelectItem for ZoneItem {
    type Value = String;

    fn title(&self) -> SharedString {
        SharedString::from(self.zone.name.clone())
    }

    fn value(&self) -> &Self::Value {
        &self.zone.id
    }
}

// Wrapper for DNS Record Type to implement SelectItem
#[derive(Clone)]
pub struct RecordTypeItem {
    pub record_type: DnsRecordType,
}

impl SelectItem for RecordTypeItem {
    type Value = DnsRecordType;

    fn title(&self) -> SharedString {
        SharedString::from(self.record_type.as_str())
    }

    fn value(&self) -> &Self::Value {
        &self.record_type
    }
}

// Wrapper for AppearanceMode to implement SelectItem
#[derive(Clone)]
pub struct AppearanceModeItem {
    pub mode: AppearanceMode,
}

impl SelectItem for AppearanceModeItem {
    type Value = AppearanceMode;

    fn title(&self) -> SharedString {
        SharedString::from(self.mode.label())
    }

    fn value(&self) -> &Self::Value {
        &self.mode
    }
}

// Main application state
pub struct App {
    pub page: Page,
    pub client: Option<CloudflareClient>,
    pub zones: Vec<Zone>,
    pub selected_zone_index: Option<usize>,
    pub dns_records: Vec<DnsRecord>,
    pub loading: bool,
    pub error: Option<String>,

    // UI state
    pub token_input: Entity<InputState>,
    pub zone_select: Entity<SelectState<Vec<ZoneItem>>>,

    // Record editor state
    pub editing_record: Option<DnsRecord>,
    pub record_type_select: Entity<SelectState<Vec<RecordTypeItem>>>,
    pub record_name_input: Entity<InputState>,
    pub record_content_input: Entity<InputState>,
    pub record_ttl_input: Entity<InputState>,
    pub record_priority_input: Entity<InputState>,
    pub record_proxied: bool,
    pub record_comment_input: Entity<InputState>,

    // Settings
    pub settings_token_input: Entity<InputState>,
    pub appearance_mode: AppearanceMode,
    pub appearance_mode_select: Entity<SelectState<Vec<AppearanceModeItem>>>,

    // DNS list scroll handle
    pub dns_list_scroll_handle: VirtualListScrollHandle,
}

impl App {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Check if we have a stored token
        let has_token = storage::has_token();
        let initial_page = if has_token {
            Page::Dashboard
        } else {
            Page::TokenSetup
        };

        // Create input states
        let token_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Paste your Cloudflare API token here...")
        });

        let zone_select = cx.new(|cx| SelectState::new(Vec::<ZoneItem>::new(), None, window, cx));

        // Create record type items
        let record_type_items: Vec<RecordTypeItem> = DnsRecordType::all()
            .iter()
            .map(|t| RecordTypeItem { record_type: *t })
            .collect();

        let record_type_select = cx.new(|cx| {
            SelectState::new(
                record_type_items,
                Some(gpui_component::IndexPath::new(0)),
                window,
                cx,
            )
        });

        let record_name_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Record name (e.g., www)"));

        let record_content_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Content (e.g., 192.168.1.1)"));

        let record_ttl_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder("TTL (1 = auto)");
            state.set_value("1", window, cx);
            state
        });

        let record_priority_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Priority (for MX/SRV)"));

        let record_comment_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Comment (optional)"));

        let settings_token_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Enter new API token..."));

        // Load saved appearance mode or default to Auto
        let saved_appearance_mode = storage::get_appearance_mode()
            .ok()
            .flatten()
            .map(|s| AppearanceMode::parse(&s))
            .unwrap_or_default();

        // Create appearance mode selector items
        let appearance_mode_items = vec![
            AppearanceModeItem {
                mode: AppearanceMode::Auto,
            },
            AppearanceModeItem {
                mode: AppearanceMode::Light,
            },
            AppearanceModeItem {
                mode: AppearanceMode::Dark,
            },
        ];

        // Find the index of the saved appearance mode
        let selected_appearance_index = appearance_mode_items
            .iter()
            .position(|item| item.mode == saved_appearance_mode)
            .map(gpui_component::IndexPath::new);

        let appearance_mode_select = cx.new(|cx| {
            SelectState::new(appearance_mode_items, selected_appearance_index, window, cx)
        });

        let mut app = Self {
            page: initial_page,
            client: None,
            zones: Vec::new(),
            selected_zone_index: None,
            dns_records: Vec::new(),
            loading: false,
            error: None,
            token_input,
            zone_select,
            editing_record: None,
            record_type_select,
            record_name_input,
            record_content_input,
            record_ttl_input,
            record_priority_input,
            record_proxied: false,
            record_comment_input,
            settings_token_input,
            appearance_mode: saved_appearance_mode,
            appearance_mode_select,
            dns_list_scroll_handle: VirtualListScrollHandle::new(),
        };

        // Apply the initial theme based on saved appearance mode
        app.apply_theme(window, cx);

        // Subscribe to zone selection changes
        cx.subscribe_in(
            &app.zone_select,
            window,
            |this, _, event: &SelectEvent<Vec<ZoneItem>>, window, cx| {
                if let SelectEvent::Confirm(Some(zone_id)) = event {
                    // Find the index of the selected zone by id
                    if let Some(index) = this.zones.iter().position(|z| &z.id == zone_id)
                        && this.selected_zone_index != Some(index)
                    {
                        this.selected_zone_index = Some(index);
                        this.editing_record = None;
                        this.load_dns_records(window, cx);
                    }
                }
            },
        )
        .detach();

        // Subscribe to appearance mode selection changes
        cx.subscribe_in(
            &app.appearance_mode_select,
            window,
            |this, _, event: &SelectEvent<Vec<AppearanceModeItem>>, window, cx| {
                if let SelectEvent::Confirm(Some(mode)) = event {
                    this.set_appearance_mode(*mode, window, cx);
                }
            },
        )
        .detach();

        // If we have a token, initialize the client and load zones
        if has_token && let Ok(Some(token)) = storage::get_token() {
            app.client = Some(CloudflareClient::new(token));
            app.load_zones(window, cx);
        }

        app
    }

    fn load_zones(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };

        self.loading = true;
        self.error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = client.list_zones().await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(zones) => {
                            // Update select items
                            let zone_items: Vec<ZoneItem> =
                                zones.iter().map(|z| ZoneItem { zone: z.clone() }).collect();
                            this.zone_select.update(cx, |state, cx| {
                                state.set_items(zone_items, window, cx);
                                if !zones.is_empty() {
                                    state.set_selected_index(
                                        Some(gpui_component::IndexPath::new(0)),
                                        window,
                                        cx,
                                    );
                                }
                            });
                            this.zones = zones;
                            if !this.zones.is_empty() && this.selected_zone_index.is_none() {
                                this.selected_zone_index = Some(0);
                                this.load_dns_records(window, cx);
                            }
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to load zones: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn load_dns_records(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(zone_index) = self.selected_zone_index else {
            return;
        };
        let Some(zone) = self.zones.get(zone_index) else {
            return;
        };

        let zone_id = zone.id.clone();
        self.loading = true;
        self.error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = client.list_dns_records(&zone_id).await;
            cx.update(|_window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(records) => {
                            this.dns_records = records;
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to load DNS records: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn save_token(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let token = self.token_input.read(cx).value().to_string();
        if token.is_empty() {
            self.error = Some("Please enter an API token".to_string());
            cx.notify();
            return;
        }

        self.loading = true;
        self.error = None;
        cx.notify();

        let client = CloudflareClient::new(token.clone());

        cx.spawn_in(window, async move |this, cx| {
            let result = client.verify_token().await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(true) => {
                            // Token is valid, store it
                            if let Err(e) = storage::store_token(&token) {
                                this.error = Some(format!("Failed to store token: {}", e));
                            } else {
                                this.client = Some(client);
                                this.page = Page::Dashboard;
                                this.load_zones(window, cx);
                            }
                        }
                        Ok(false) => {
                            this.error = Some("Token is not active".to_string());
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to verify token: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn update_token_from_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let token = self.settings_token_input.read(cx).value().to_string();
        if token.is_empty() {
            self.error = Some("Please enter an API token".to_string());
            cx.notify();
            return;
        }

        self.loading = true;
        self.error = None;
        cx.notify();

        let client = CloudflareClient::new(token.clone());

        cx.spawn_in(window, async move |this, cx| {
            let result = client.verify_token().await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(true) => {
                            if let Err(e) = storage::store_token(&token) {
                                this.error = Some(format!("Failed to store token: {}", e));
                            } else {
                                this.client = Some(client);
                                this.zones.clear();
                                this.dns_records.clear();
                                this.selected_zone_index = None;
                                this.settings_token_input.update(cx, |input, cx| {
                                    input.set_value("", window, cx);
                                });
                                this.page = Page::Dashboard;
                                this.load_zones(window, cx);
                                window.push_notification(
                                    Notification::success("API token updated successfully"),
                                    cx,
                                );
                            }
                        }
                        Ok(false) => {
                            this.error = Some("Token is not active".to_string());
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to verify token: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn clear_token(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = storage::delete_token() {
            self.error = Some(format!("Failed to delete token: {}", e));
            cx.notify();
            return;
        }

        self.client = None;
        self.zones.clear();
        self.dns_records.clear();
        self.selected_zone_index = None;
        self.page = Page::TokenSetup;
        cx.notify();
    }

    pub fn create_record(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(zone_index) = self.selected_zone_index else {
            return;
        };
        let Some(zone) = self.zones.get(zone_index) else {
            return;
        };

        let record_type = self
            .record_type_select
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(DnsRecordType::A);
        let name = self.record_name_input.read(cx).value().to_string();
        let content = self.record_content_input.read(cx).value().to_string();
        let ttl: u32 = self.record_ttl_input.read(cx).value().parse().unwrap_or(1);
        let priority: Option<u16> = self.record_priority_input.read(cx).value().parse().ok();
        let comment = {
            let c = self.record_comment_input.read(cx).value().to_string();
            if c.is_empty() { None } else { Some(c) }
        };

        // Validate
        if name.is_empty() {
            self.error = Some("Record name is required".to_string());
            cx.notify();
            return;
        }
        if content.is_empty() {
            self.error = Some("Content is required".to_string());
            cx.notify();
            return;
        }
        if let Err(e) = record_type.validate_content(&content) {
            self.error = Some(e.to_string());
            cx.notify();
            return;
        }

        let zone_id = zone.id.clone();
        let record = CreateDnsRecord {
            record_type,
            name,
            content,
            ttl,
            proxied: if record_type.is_proxiable() {
                Some(self.record_proxied)
            } else {
                None
            },
            priority,
            comment,
        };

        self.loading = true;
        self.error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = client.create_dns_record(&zone_id, &record).await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(_) => {
                            this.clear_record_form(window, cx);
                            this.load_dns_records(window, cx);
                            window.push_notification(
                                Notification::success("DNS record created successfully"),
                                cx,
                            );
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to create record: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn update_record(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(zone_index) = self.selected_zone_index else {
            return;
        };
        let Some(zone) = self.zones.get(zone_index) else {
            return;
        };
        let Some(editing) = &self.editing_record else {
            return;
        };

        let record_type = self
            .record_type_select
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(DnsRecordType::A);
        let name = self.record_name_input.read(cx).value().to_string();
        let content = self.record_content_input.read(cx).value().to_string();
        let ttl: u32 = self.record_ttl_input.read(cx).value().parse().unwrap_or(1);
        let priority: Option<u16> = self.record_priority_input.read(cx).value().parse().ok();
        let comment = {
            let c = self.record_comment_input.read(cx).value().to_string();
            if c.is_empty() { None } else { Some(c) }
        };

        // Validate
        if let Err(e) = record_type.validate_content(&content) {
            self.error = Some(e.to_string());
            cx.notify();
            return;
        }

        let zone_id = zone.id.clone();
        let record_id = editing.id.clone();
        let record = UpdateDnsRecord {
            record_type: Some(record_type),
            name: Some(name),
            content: Some(content),
            ttl: Some(ttl),
            proxied: if record_type.is_proxiable() {
                Some(self.record_proxied)
            } else {
                None
            },
            priority,
            comment,
        };

        self.loading = true;
        self.error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = client
                .update_dns_record(&zone_id, &record_id, &record)
                .await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(_) => {
                            this.editing_record = None;
                            this.clear_record_form(window, cx);
                            this.load_dns_records(window, cx);
                            window.push_notification(
                                Notification::success("DNS record updated successfully"),
                                cx,
                            );
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to update record: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn delete_record(
        &mut self,
        record_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(client) = self.client.clone() else {
            return;
        };
        let Some(zone_index) = self.selected_zone_index else {
            return;
        };
        let Some(zone) = self.zones.get(zone_index) else {
            return;
        };

        let zone_id = zone.id.clone();
        self.loading = true;
        self.error = None;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = client.delete_dns_record(&zone_id, &record_id).await;
            cx.update(|window, cx| {
                this.update(cx, |this, cx| {
                    this.loading = false;
                    match result {
                        Ok(_) => {
                            this.load_dns_records(window, cx);
                            window.push_notification(
                                Notification::success("DNS record deleted successfully"),
                                cx,
                            );
                        }
                        Err(e) => {
                            this.error = Some(format!("Failed to delete record: {}", e));
                        }
                    }
                    cx.notify();
                })
                .ok();
            })
            .ok();
        })
        .detach();
    }

    pub fn edit_record(&mut self, record: DnsRecord, window: &mut Window, cx: &mut Context<Self>) {
        // Find the index of the record type
        let type_index = DnsRecordType::all()
            .iter()
            .position(|t| *t == record.record_type)
            .unwrap_or(0);

        self.record_type_select.update(cx, |state, cx| {
            state.set_selected_index(Some(gpui_component::IndexPath::new(type_index)), window, cx);
        });

        self.record_name_input.update(cx, |input, cx| {
            input.set_value(&record.name, window, cx);
        });

        self.record_content_input.update(cx, |input, cx| {
            input.set_value(&record.content, window, cx);
        });

        self.record_ttl_input.update(cx, |input, cx| {
            input.set_value(record.ttl.to_string(), window, cx);
        });

        if let Some(priority) = record.priority {
            self.record_priority_input.update(cx, |input, cx| {
                input.set_value(priority.to_string(), window, cx);
            });
        } else {
            self.record_priority_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
            });
        }

        if let Some(comment) = &record.comment {
            self.record_comment_input.update(cx, |input, cx| {
                input.set_value(comment, window, cx);
            });
        } else {
            self.record_comment_input.update(cx, |input, cx| {
                input.set_value("", window, cx);
            });
        }

        self.record_proxied = record.proxied;
        self.editing_record = Some(record);
        cx.notify();
    }

    pub fn clear_record_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_record = None;
        self.record_proxied = false;

        self.record_type_select.update(cx, |state, cx| {
            state.set_selected_index(Some(gpui_component::IndexPath::new(0)), window, cx);
        });
        self.record_name_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.record_content_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.record_ttl_input.update(cx, |input, cx| {
            input.set_value("1", window, cx);
        });
        self.record_priority_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.record_comment_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
    }

    pub fn apply_theme(&self, window: &mut Window, cx: &mut gpui::App) {
        match self.appearance_mode {
            AppearanceMode::Auto => {
                Theme::sync_system_appearance(Some(window), cx);
            }
            AppearanceMode::Light => {
                Theme::change(ThemeMode::Light, Some(window), cx);
            }
            AppearanceMode::Dark => {
                Theme::change(ThemeMode::Dark, Some(window), cx);
            }
        }
    }

    pub fn set_appearance_mode(
        &mut self,
        mode: AppearanceMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.appearance_mode = mode;

        // Save to storage
        if let Err(e) = storage::store_appearance_mode(mode.as_str()) {
            self.error = Some(format!("Failed to save appearance mode: {}", e));
        }

        // Apply the theme
        self.apply_theme(window, cx);
        cx.notify();
    }
}

impl Render for App {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(match self.page {
                Page::TokenSetup => ui::render_token_setup(self, window, cx).into_any_element(),
                Page::Dashboard => ui::render_dashboard(self, window, cx).into_any_element(),
                Page::Settings => ui::render_settings(self, window, cx).into_any_element(),
            })
            .children(Root::render_notification_layer(window, cx))
    }
}

#[tokio::main]
async fn main() {
    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1200.), px(800.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Cloudflare DNS Manager".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            let app_view = cx.new(|cx| App::new(window, cx));
            cx.new(|cx| Root::new(app_view.clone(), window, cx))
        })
        .ok();
    });
}
