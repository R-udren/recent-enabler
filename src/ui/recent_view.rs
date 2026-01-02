//! Recent Files card UI.

use crate::domain::{CheckSeverity, RecentInfo, RecentStatus};
use crate::ui::components;
use iced::widget::{button, column, container, text};
use iced::{Color, Element};

pub fn view<M: Clone + 'static>(
    info: Option<&RecentInfo>,
    on_enable: M,
    on_open: M,
) -> Element<'_, M> {
    let Some(info) = info else {
        return container(text("Loading...").size(16)).padding(20).into();
    };

    let status_text = match info.status {
        RecentStatus::FullyEnabled => "ENABLED",
        RecentStatus::PartiallyEnabled => "PARTIAL",
        RecentStatus::FullyDisabled => "DISABLED",
        RecentStatus::PolicyBlocked => "BLOCKED",
    };
    let is_ok = info.status == RecentStatus::FullyEnabled;

    fn fmt_time(t: Option<std::time::SystemTime>) -> String {
        t.map(|s| {
            let dt: chrono::DateTime<chrono::Local> = s.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        })
        .unwrap_or_else(|| "N/A".into())
    }

    let mut content = column![
        components::card_header("Recent Files", on_open),
        components::info_row("Status:", components::status_text(status_text, is_ok)),
        components::info_row("Files:", text(info.files.count.to_string()).size(16)),
        components::info_row("Oldest:", text(fmt_time(info.files.oldest)).size(14)),
        components::info_row("Newest:", text(fmt_time(info.files.newest)).size(14)),
    ]
    .spacing(10);

    // Show critical/important issues
    let issues: Vec<_> = info
        .checks
        .iter()
        .filter(|c| {
            !c.is_ok()
                && matches!(
                    c.severity,
                    CheckSeverity::Important | CheckSeverity::Critical
                )
        })
        .map(|c| c.name.clone())
        .collect();

    if !issues.is_empty() {
        content = content.push(
            text(format!("Issues: {}", issues.join(", ")))
                .size(12)
                .color(Color::from_rgb(1.0, 0.7, 0.3)),
        );
    }

    if !is_ok && info.status != RecentStatus::PolicyBlocked {
        content = content.push(button("Enable").on_press(on_enable).padding(10));
    }

    components::card_container(
        content,
        Color::from_rgb(0.15, 0.2, 0.25),
        Color::from_rgb(0.3, 0.4, 0.5),
    )
    .into()
}
