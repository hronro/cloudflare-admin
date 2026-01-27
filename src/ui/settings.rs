use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Disableable,
    button::{Button, ButtonVariants},
    h_flex,
    input::Input,
    select::Select,
    v_flex,
};

use crate::{App, AppearanceModeItem, Page};

pub fn render_settings(
    app: &mut App,
    _window: &mut Window,
    cx: &mut Context<App>,
) -> impl IntoElement {
    let is_loading = app.loading;
    let error = app.error.clone();

    v_flex()
        .size_full()
        .child(
            // Header
            h_flex()
                .w_full()
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(cx.theme().border)
                .items_center()
                .gap_3()
                .child(
                    Button::new("back")
                        .ghost()
                        .icon(gpui_component::IconName::ArrowLeft)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.page = Page::Dashboard;
                            this.error = None;
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Settings"),
                ),
        )
        .child(
            v_flex()
                .p_6()
                .gap_6()
                .max_w(px(600.))
                // API Token section
                .child(
                    v_flex()
                        .gap_4()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().font_weight(FontWeight::SEMIBOLD).child("API Token"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Update your Cloudflare API token"),
                                ),
                        )
                        .child(Input::new(&app.settings_token_input))
                        .map(|this| {
                            if let Some(err) = error {
                                this.child(
                                    div()
                                        .px_3()
                                        .py_2()
                                        .rounded_md()
                                        .bg(cx.theme().danger.opacity(0.1))
                                        .text_color(cx.theme().danger)
                                        .text_sm()
                                        .child(err),
                                )
                            } else {
                                this
                            }
                        })
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("update-token")
                                        .primary()
                                        .label(if is_loading {
                                            "Verifying..."
                                        } else {
                                            "Update Token"
                                        })
                                        .disabled(is_loading)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.update_token_from_settings(window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("clear-token")
                                        .danger()
                                        .label("Clear Token")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.clear_token(cx);
                                        })),
                                ),
                        ),
                )
                // Appearance section
                .child(
                    v_flex()
                        .gap_4()
                        .pt_4()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().font_weight(FontWeight::SEMIBOLD).child("Appearance"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Choose your preferred color theme"),
                                ),
                        )
                        .child(
                            Select::<Vec<AppearanceModeItem>>::new(&app.appearance_mode_select)
                                .w(px(200.)),
                        ),
                )
                // About section
                .child(
                    v_flex()
                        .gap_2()
                        .pt_4()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("About"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Cloudflare DNS Manager v0.1.0"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Built with GPUI and gpui-component"),
                        ),
                ),
        )
}
