//! Prefetch card UI.

use crate::domain::PrefetchInfo;
use crate::ui::components;
use iced::widget::{button, column, text};
use iced::{Color, Element};

pub fn view<M: Clone + 'static>(
    info: Option<&PrefetchInfo>,
    is_admin: bool,
    on_enable: M,
    on_open: M,
    on_restart: M,
) -> Element<'_, M> {
    let Some(info) = info else {
        return components::card_container(
            text("Loading...").size(16),
            Color::from_rgb(0.15, 0.25, 0.2),
            Color::from_rgb(0.3, 0.5, 0.4),
        )
        .into();
    };

    fn fmt_time(t: Option<std::time::SystemTime>) -> String {
        t.map(|s| {
            let dt: chrono::DateTime<chrono::Local> = s.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "N/A".into())
    }

    let mut content = column![
        components::card_header("Prefetch", on_open),
        components::info_row(
            "Service:",
            components::status_text(
                match info.service_state {
                    crate::domain::ServiceState::Running => "RUNNING",
                    crate::domain::ServiceState::Stopped => "STOPPED",
                    crate::domain::ServiceState::Unknown => "UNKNOWN",
                },
                info.service_state == crate::domain::ServiceState::Running
            )
        ),
        components::info_row("Files:", text(info.files.count.to_string()).size(16)),
        components::info_row("Oldest:", text(fmt_time(info.files.oldest)).size(14)),
        components::info_row("Newest:", text(fmt_time(info.files.newest)).size(14)),
    ]
    .spacing(10);

    if let Some(err) = &info.error {
        content = content.push(text(err).size(12).color(Color::from_rgb(1.0, 0.7, 0.3)));
    }

    if !info.is_ok() {
        if is_admin {
            content = content.push(button("Enable").on_press(on_enable).padding(10));
        } else {
            content = content.push(components::admin_warning(on_restart));
        }
    }

    components::card_container(
        content,
        Color::from_rgb(0.15, 0.25, 0.2),
        Color::from_rgb(0.3, 0.5, 0.4),
    )
    .into()
}
