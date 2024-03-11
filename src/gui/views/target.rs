use iced::widget::{self, button, text, Column};
use iced::Element;

use super::super::{lang, style, utils, App};

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
                style::ButtonStyleSheet::new().set_background(
                    iced::Color::from_rgb8(232, 205, 64),
                    iced::Color::from_rgb8(242, 225, 84),
                ),
            ),
    );

    if let Some(target) = match app.db.get_setting("target_path") {
        Ok(value) => value,
        Err(error) => {
            utils::error_popup(&utils::error_chain_string(error));
            None
        }
    } {
        col = col.push(text(target));
    }

    if let Some(last_sync) = &app.last_sync {
        col = col.push(text(format!(
            "{}: {}",
            lang::last_sync(&app.lang),
            last_sync.timestamp().format("%d.%m.%Y %H:%M")
        )));
    }

    widget::container(col)
        .style(
            style::ContainerStyleSheet::new()
                .background(Some(iced::Background::Color(iced::Color::from_rgb8(
                    254, 234, 54,
                ))))
                .border_radius(iced::Border::with_radius(20.0)),
        )
        .padding(10)
        .width(iced::Length::Fill)
        .center_x()
        .center_y()
        .into()
}

pub(in super::super) fn update(app: &mut App, message: Message) {
    match message {
        Message::ChangeTarget => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                if let Err(error) = app.db.set_setting("target_path", path.to_str().unwrap()) {
                    utils::error_popup(&utils::error_chain_string(error));
                }
            }

            if let Err(error) = app.reload_last_sync() {
                utils::error_popup(&utils::error_chain_string(error));
            }
        }
    }
}
