//! System Restore card UI.

use crate::domain::SystemRestoreInfo;
use crate::ui::components;
use iced::widget::{button, column, row, text};
use iced::{Color, Element};

pub fn view<M: Clone + 'static>(
    info: Option<&SystemRestoreInfo>,
    is_admin: bool,
    on_enable: M,
    on_restart: M,
    on_set_freq0: M,
    on_set_disk10: M,
) -> Element<'_, M> {
    let Some(info) = info else {
        return components::card_container(
            text("Loading...").size(16),
            Color::from_rgb(0.2, 0.15, 0.25),
            Color::from_rgb(0.5, 0.3, 0.5),
        )
        .into();
    };

    fn fmt_time_opt(t: Option<u32>) -> String {
        t.map(|v| v.to_string()).unwrap_or_else(|| "N/A".into())
    }

    let mut content = column![
        text("System Restore").size(22),
        components::info_row(
            "Status:",
            components::status_text(
                if info.enabled { "ENABLED" } else { "DISABLED" },
                info.enabled
            )
        ),
        components::info_row(
            "Frequency (min):",
            text(fmt_time_opt(info.frequency_minutes)).size(14)
        ),
        components::info_row("Disk %:", text(fmt_time_opt(info.disk_percent)).size(14)),
    ]
    .spacing(10);

    if !info.enabled {
        if is_admin {
            content = content.push(button("Enable").on_press(on_enable).padding(10));
        } else {
            content = content.push(components::admin_warning(on_restart));
        }
    } else {
        // Add quick-config buttons
        content = content.push(
            row![
                button("Allow On-Demand").on_press(on_set_freq0).padding(6),
                button("Disk 10%").on_press(on_set_disk10).padding(6),
            ]
            .spacing(8),
        );
    }

    components::card_container(
        content,
        Color::from_rgb(0.2, 0.15, 0.25),
        Color::from_rgb(0.5, 0.3, 0.5),
    )
    .into()
}
