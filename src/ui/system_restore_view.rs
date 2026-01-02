//! System Restore card UI.

use crate::domain::SystemRestoreInfo;
use crate::ui::components;
use iced::widget::{button, column, text};
use iced::{Color, Element};

pub fn view<M: Clone + 'static>(
    info: Option<&SystemRestoreInfo>,
    is_admin: bool,
    on_enable: M,
    on_restart: M,
) -> Element<'_, M> {
    let Some(info) = info else {
        return components::card_container(
            text("Loading...").size(16),
            Color::from_rgb(0.2, 0.15, 0.25),
            Color::from_rgb(0.5, 0.3, 0.5),
        )
        .into();
    };

    let mut content = column![
        text("System Restore").size(22),
        components::info_row(
            "Status:",
            components::status_text(
                if info.enabled { "ENABLED" } else { "DISABLED" },
                info.enabled
            )
        ),
    ]
    .spacing(10);

    if !info.enabled {
        if is_admin {
            content = content.push(button("Enable").on_press(on_enable).padding(10));
        } else {
            content = content.push(components::admin_warning(on_restart));
        }
    }

    components::card_container(
        content,
        Color::from_rgb(0.2, 0.15, 0.25),
        Color::from_rgb(0.5, 0.3, 0.5),
    )
    .into()
}
