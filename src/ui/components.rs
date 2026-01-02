//! Reusable UI components.

use iced::widget::{button, container, row, text, Row, Text};
use iced::{Color, Element, Fill};

pub fn status_text(label: &str, is_good: bool) -> Text<'_> {
    text(label).size(18).color(if is_good {
        Color::from_rgb(0.4, 1.0, 0.4)
    } else {
        Color::from_rgb(1.0, 0.4, 0.4)
    })
}

pub fn info_row<'a, M: 'a>(label: &'a str, value: impl Into<Element<'a, M>>) -> Row<'a, M> {
    row![
        text(label)
            .size(15)
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .width(140),
        value.into()
    ]
    .spacing(10)
}

pub fn card_header<'a, M: Clone + 'a>(title: &'a str, on_open: M) -> Row<'a, M> {
    row![
        text(title).size(22),
        iced::widget::Space::new().width(Fill),
        button("Open").on_press(on_open).padding([6, 12]),
    ]
    .align_y(iced::Alignment::Center)
}

pub fn card_container<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    bg: Color,
    border: Color,
) -> container::Container<'a, M> {
    container(content)
        .padding(20)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: border,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
}

pub fn admin_warning<'a, M: Clone + 'a>(on_restart: M) -> container::Container<'a, M> {
    container(
        row![
            text("Requires admin rights").size(13).width(Fill),
            button("Restart").on_press(on_restart).padding([5, 10]),
        ]
        .spacing(8),
    )
    .padding(10)
    .style(|_| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.25, 0.2, 0.15))),
        border: iced::Border {
            color: Color::from_rgb(0.6, 0.5, 0.3),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
}
