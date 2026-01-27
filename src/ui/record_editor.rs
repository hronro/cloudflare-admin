use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Disableable, Sizable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    h_flex,
    input::Input,
    scroll::ScrollableElement,
    select::Select,
    v_flex,
};

use crate::{App, cloudflare::DnsRecordType};

pub fn render_record_editor(
    app: &mut App,
    _window: &mut Window,
    cx: &mut Context<App>,
) -> impl IntoElement {
    let is_loading = app.loading;
    let editing = app.editing_record.is_some();
    let current_record_type = app
        .record_type_select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(DnsRecordType::A);
    let error = app.error.clone();

    v_flex()
        .w(px(350.))
        .h_full()
        .border_l_1()
        .border_color(cx.theme().border)
        .p_4()
        .gap_4()
        .overflow_y_scrollbar()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().font_weight(FontWeight::SEMIBOLD).child(if editing {
                    "Edit Record"
                } else {
                    "New Record"
                }))
                .map(|this| {
                    if editing {
                        this.child(
                            Button::new("cancel-edit")
                                .ghost()
                                .small()
                                .label("Cancel")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.clear_record_form(window, cx);
                                    cx.notify();
                                })),
                        )
                    } else {
                        this
                    }
                }),
        )
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
            v_flex()
                .gap_3()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Type"),
                        )
                        .child(Select::new(&app.record_type_select).w_full()),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Name"),
                        )
                        .child(Input::new(&app.record_name_input)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Content"),
                        )
                        .child(Input::new(&app.record_content_input)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().font_weight(FontWeight::MEDIUM).child("TTL"))
                        .child(Input::new(&app.record_ttl_input)),
                )
                .map(|this| {
                    if current_record_type.requires_priority() {
                        this.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Priority"),
                                )
                                .child(Input::new(&app.record_priority_input)),
                        )
                    } else {
                        this
                    }
                })
                .map(|this| {
                    if current_record_type.is_proxiable() {
                        this.child(
                            Checkbox::new("proxied")
                                .label("Proxied through Cloudflare")
                                .checked(app.record_proxied)
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    this.record_proxied = *checked;
                                    cx.notify();
                                })),
                        )
                    } else {
                        this
                    }
                })
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Comment"),
                        )
                        .child(Input::new(&app.record_comment_input)),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("save-record")
                                .primary()
                                .flex_1()
                                .label(if editing {
                                    "Update Record"
                                } else {
                                    "Create Record"
                                })
                                .disabled(is_loading)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if this.editing_record.is_some() {
                                        this.update_record(window, cx);
                                    } else {
                                        this.create_record(window, cx);
                                    }
                                })),
                        )
                        .map(|this| {
                            if editing {
                                this.child(
                                    Button::new("delete-record")
                                        .danger()
                                        .icon(gpui_component::IconName::Delete)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            if let Some(record) = &this.editing_record {
                                                let record_id = record.id.clone();
                                                this.delete_record(record_id, window, cx);
                                            }
                                        })),
                                )
                            } else {
                                this
                            }
                        }),
                ),
        )
}
