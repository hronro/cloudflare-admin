use gpui::prelude::*;
use gpui::{Context, FontWeight, IntoElement, Window, div, px};
use gpui_component::{
    ActiveTheme, Disableable,
    button::{Button, ButtonVariants},
    input::Input,
    v_flex,
};

use crate::App;

pub fn render_token_setup(
    app: &mut App,
    _window: &mut Window,
    cx: &mut Context<App>,
) -> impl IntoElement {
    let is_loading = app.loading;

    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .p_8()
        .child(
            v_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .child("Cloudflare DNS Manager"),
                )
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("Enter your Cloudflare API token to get started"),
                ),
        )
        .child(
            v_flex()
                .w(px(400.))
                .gap_4()
                .child(Input::new(&app.token_input))
                .map(|this| {
                    if let Some(error) = app.error.clone() {
                        this.child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(cx.theme().danger.opacity(0.1))
                                .text_color(cx.theme().danger)
                                .text_sm()
                                .child(error),
                        )
                    } else {
                        this
                    }
                })
                .child(
                    Button::new("save-token")
                        .primary()
                        .w_full()
                        .label(if is_loading {
                            "Verifying..."
                        } else {
                            "Save Token"
                        })
                        .disabled(is_loading)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.save_token(window, cx);
                        })),
                ),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Your token will be stored securely in your system's keychain."),
        )
}
