use std::rc::Rc;

use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, Pixels, SharedString, Size, Window, div, px, size};
use gpui_component::{ActiveTheme, h_flex, orange_500, scroll::Scrollbar, v_flex, v_virtual_list};

use crate::App;

const ITEM_HEIGHT: Pixels = px(56.);

pub fn render_dns_list(
    app: &mut App,
    _window: &mut Window,
    cx: &mut Context<App>,
) -> impl IntoElement {
    let records = app.dns_records.clone();
    let records_count = records.len();
    let editing_id = app.editing_record.as_ref().map(|r| r.id.clone());
    let scroll_handle = &app.dns_list_scroll_handle;

    // Pre-calculate item sizes for virtual list
    let item_sizes: Rc<Vec<Size<Pixels>>> = Rc::new(
        (0..records_count)
            .map(|_| size(px(0.), ITEM_HEIGHT))
            .collect(),
    );

    let border_color = cx.theme().border;
    let accent_color = cx.theme().accent;
    let primary_color = cx.theme().primary;
    let muted_foreground = cx.theme().muted_foreground;

    div()
        .flex_1()
        .overflow_hidden()
        .border_1()
        .border_color(border_color)
        .rounded_md()
        .map(|this| {
            if records.is_empty() {
                this.child(
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(muted_foreground)
                        .child("No DNS records found"),
                )
            } else {
                this.child(
                    div()
                        .size_full()
                        .relative()
                        .child(
                            v_virtual_list(
                                cx.entity(),
                                "dns-records-list",
                                item_sizes,
                                move |app, visible_range, _window, cx| {
                                    visible_range
                                        .map(|ix| {
                                            let record = &app.dns_records[ix];
                                            let record_clone = record.clone();
                                            let is_selected =
                                                editing_id.as_ref() == Some(&record.id);

                                            div()
                                                .id(SharedString::from(record.id.clone()))
                                                .w_full()
                                                .h(ITEM_HEIGHT)
                                                .px_3()
                                                .flex()
                                                .items_center()
                                                .border_b_1()
                                                .border_color(border_color)
                                                .cursor_pointer()
                                                .map(|this| {
                                                    if is_selected {
                                                        this.bg(accent_color)
                                                    } else {
                                                        this
                                                    }
                                                })
                                                .hover(|this| this.bg(accent_color.opacity(0.5)))
                                                .on_click(cx.listener(
                                                    move |this, _, window, cx| {
                                                        this.edit_record(
                                                            record_clone.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                    },
                                                ))
                                                .child(
                                                    h_flex()
                                                        .w_full()
                                                        .items_center()
                                                        .gap_3()
                                                        .child(
                                                            div()
                                                                .w(px(50.))
                                                                .px_2()
                                                                .py_1()
                                                                .rounded_sm()
                                                                .bg(primary_color.opacity(0.1))
                                                                .text_xs()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(primary_color)
                                                                .child(record.record_type.as_str()),
                                                        )
                                                        .child(
                                                            v_flex()
                                                                .flex_1()
                                                                .overflow_hidden()
                                                                .child(
                                                                    div()
                                                                        .text_sm()
                                                                        .font_weight(
                                                                            FontWeight::MEDIUM,
                                                                        )
                                                                        .truncate()
                                                                        .child(record.name.clone()),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(
                                                                            muted_foreground,
                                                                        )
                                                                        .truncate()
                                                                        .child(
                                                                            record.content.clone(),
                                                                        ),
                                                                ),
                                                        )
                                                        .child(
                                                            h_flex()
                                                                .gap_2()
                                                                .items_center()
                                                                .map(|this| {
                                                                    if record.proxied {
                                                                        this.child(
                                                                            div()
                                                                                .px_1()
                                                                                .py_px()
                                                                                .rounded_sm()
                                                                                .bg(orange_500()
                                                                                    .opacity(0.2))
                                                                                .text_xs()
                                                                                .text_color(
                                                                                    orange_500(),
                                                                                )
                                                                                .child("Proxied"),
                                                                        )
                                                                    } else {
                                                                        this
                                                                    }
                                                                })
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(
                                                                            muted_foreground,
                                                                        )
                                                                        .child(
                                                                            if record.ttl == 1 {
                                                                                "Auto".to_string()
                                                                            } else {
                                                                                format!(
                                                                                    "{}s",
                                                                                    record.ttl
                                                                                )
                                                                            },
                                                                        ),
                                                                ),
                                                        ),
                                                )
                                        })
                                        .collect()
                                },
                            )
                            .size_full()
                            .track_scroll(scroll_handle),
                        )
                        .child(
                            // Scrollbar overlay - must be absolutely positioned over the entire area
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .bottom_0()
                                .child(Scrollbar::vertical(scroll_handle)),
                        ),
                )
            }
        })
}
