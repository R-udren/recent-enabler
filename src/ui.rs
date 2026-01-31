use iced::widget::{button, column, container, row, space, text};
use iced::{Color, Element, Fill};
use std::time::SystemTime;

pub fn format_time(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.format("%d.%m.%Y %H:%M").to_string()
}

pub fn time_ago(time: SystemTime) -> String {
    let now = SystemTime::now();
    if let Ok(duration) = now.duration_since(time) {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let days = total_secs / 86400;

        if hours < 1 {
            let mins = total_secs / 60;
            if mins < 1 {
                "только что".to_string()
            } else {
                format!("{} мин. назад", mins)
            }
        } else if hours < 24 {
            format!("{} ч. назад", hours)
        } else if days == 1 {
            "1 день назад".to_string()
        } else {
            format!("{} дн. назад", days)
        }
    } else {
        "в будущем".to_string()
    }
}

pub fn label_text(label: &str) -> text::Text<'_> {
    text(label).size(15).color(Color::from_rgb(0.7, 0.7, 0.7))
}

pub fn value_text(value: impl ToString) -> text::Text<'static> {
    text(value.to_string()).size(18)
}

pub fn status_text(value: &str, is_active: bool) -> text::Text<'_> {
    text(value).size(18).color(if is_active {
        Color::from_rgb(0.4, 1.0, 0.4)
    } else {
        Color::from_rgb(1.0, 0.4, 0.4)
    })
}

pub fn info_row<'a, M: 'a>(
    label: &'a str,
    value: impl Into<Element<'a, M>>,
) -> iced::widget::Row<'a, M> {
    row![label_text(label).width(160), value.into()].spacing(10)
}

pub fn card_header<'a, M: Clone + 'a>(title: &'a str, on_open: M) -> iced::widget::Row<'a, M> {
    row![
        text(title).size(22),
        space().width(Fill),
        button("Открыть папку").on_press(on_open).padding([6, 12]),
    ]
    .align_y(iced::Alignment::Center)
}

pub fn card_style(_theme: &iced::Theme, bg_color: Color, border_color: Color) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(bg_color)),
        border: iced::Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn warning_box<'a, M: Clone + 'static>(
    message: &'a str,
    on_restart: M,
) -> container::Container<'a, M> {
    container(
        row![
            text(message).size(13).width(Fill),
            restart_button(on_restart).padding([5, 10]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding(10)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.25, 0.2, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.6, 0.5, 0.3),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
}

pub fn restart_button<M: Clone>(on_press: M) -> button::Button<'static, M> {
    button("Перезапустить")
        .on_press(on_press)
        .style(|_theme, status| {
            let base_color = match status {
                button::Status::Active => Color::from_rgb(0.4, 0.3, 0.2),
                button::Status::Hovered => Color::from_rgb(0.5, 0.4, 0.3),
                button::Status::Pressed => Color::from_rgb(0.35, 0.25, 0.15),
                _ => Color::from_rgb(0.4, 0.3, 0.2),
            };
            button::Style {
                background: Some(iced::Background::Color(base_color)),
                text_color: Color::WHITE,
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
}

pub fn file_info_rows<'a, M: 'a>(
    oldest: &Option<SystemTime>,
    newest: &Option<SystemTime>,
) -> iced::widget::Column<'a, M> {
    let mut col = column![].spacing(10);

    if let Some(time) = oldest {
        let display = format!("{} ({})", format_time(*time), time_ago(*time));
        col = col.push(info_row("Самый старый:", text(display).size(14)));
    }

    if let Some(time) = newest {
        let display = format!("{} ({})", format_time(*time), time_ago(*time));
        col = col.push(info_row("Самый новый:", text(display).size(14)));
    }

    col
}
