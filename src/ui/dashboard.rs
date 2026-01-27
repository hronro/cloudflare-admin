use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    select::Select,
    v_flex,
};

use super::{render_dns_list, render_record_editor};
use crate::{App, Page};

pub fn render_dashboard(
    app: &mut App,
    window: &mut Window,
    cx: &mut Context<App>,
) -> impl IntoElement {
    let is_loading = app.loading;
    let dns_records = app.dns_records.clone();

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
                .justify_between()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Cloudflare DNS Manager"),
                        )
                        .map(|this| {
                            if is_loading {
                                this.child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Loading..."),
                                )
                            } else {
                                this
                            }
                        }),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Select::new(&app.zone_select)
                                .w(px(250.))
                                .placeholder("Select a domain..."),
                        )
                        .child(
                            Button::new("settings")
                                .ghost()
                                .icon(gpui_component::IconName::Settings)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.page = Page::Settings;
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            // Main content - horizontal split
            h_flex()
                .flex_1()
                .overflow_hidden()
                .child(
                    // DNS Records list (left panel)
                    v_flex()
                        .flex_1()
                        .h_full()
                        .p_4()
                        .gap_2()
                        .overflow_hidden()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(format!("DNS Records ({})", dns_records.len())),
                                )
                                .child(
                                    Button::new("refresh")
                                        .ghost()
                                        .small()
                                        .icon(gpui_component::IconName::Redo)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.load_dns_records(window, cx);
                                        })),
                                ),
                        )
                        .child(render_dns_list(app, window, cx)),
                )
                .child(render_record_editor(app, window, cx)),
        )
}
