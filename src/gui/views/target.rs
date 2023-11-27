use iced::widget::{self, button, text, Column};
use iced::Element;

use super::super::{lang, style, App};

#[derive(Debug, Clone)]
pub(in super::super) enum Message {
    ChangeTarget,
}

pub(in super::super) fn view(app: &App) -> Element<'_, Message> {
    let mut col = Column::new().align_items(iced::Alignment::Center);

    col = col.push(
        button(lang::set_target(&app.lang))
            .on_press_maybe({
                if app.is_currently_syncing() {
                    None
                } else {
                    Some(Message::ChangeTarget)
                }
            })
            .style(
                style::ButtonStyleSheet::new()
                    .set_background(
                        iced::Color::from_rgb8(232, 205, 64),
                        iced::Color::from_rgb8(242, 225, 84),
                    )
                    .into(),
            ),
    );

    if let Some(target) = app.db.get_setting("target_path").unwrap() {
        col = col.push(text(target));
    }

    widget::container(col)
        .style(
            style::ContainerStyleSheet::new()
                .background(Some(iced::Background::Color(iced::Color::from_rgb8(
                    254, 234, 54,
                ))))
                .border_radius(iced::BorderRadius::from(20.0)),
        )
        .padding(10)
        .into()
}

pub(in super::super) fn update(app: &mut App, message: Message) {
    match message {
        Message::ChangeTarget => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.db
                    .set_setting("target_path", path.to_str().unwrap())
                    .unwrap();
            }
        }
    }
}
