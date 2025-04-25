use iced::Element;
use iced::widget::{self, Column, button, text};

use super::super::{App, lang, utils};

#[derive(Debug, Clone)]
pub(in super::super) enum Message {
    ChangeTarget,
}

pub(in super::super) fn view(app: &App) -> Element<'_, Message> {
    let mut col = Column::new().align_x(iced::Alignment::Center);

    col = col.push(
        button(lang::set_target(&app.lang))
            .on_press_maybe({
                if app.is_currently_syncing() {
                    None
                } else {
                    Some(Message::ChangeTarget)
                }
            })
            .style(|theme, status| {
                let mut style = widget::button::primary(theme, status);
                if status == widget::button::Status::Active {
                    style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
                        232, 205, 64,
                    )))
                } else {
                    style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
                        242, 225, 84,
                    )))
                }
                style
            }),
    );

    if let Some(target) = app.db.get_setting("target_path").unwrap_or(None) {
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
        .style(|_| widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb8(
                254, 234, 54,
            ))),
            border: iced::Border::default().rounded(20.0),
            shadow: iced::Shadow {
                color: iced::Color::from_rgb8(0, 0, 0),
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 8.0,
            },
            text_color: None,
        })
        .padding(10)
        .center(iced::Fill)
        .into()
}

pub(in super::super) fn update(app: &mut App, message: Message) -> iced::Task<Message> {
    match message {
        Message::ChangeTarget => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                if let Err(error) = app.db.set_setting("target_path", path.to_str().unwrap()) {
                    return iced::Task::future(utils::async_error_popup(
                        &utils::error_chain_string(error),
                    ))
                    .discard();
                }
            }

            if let Err(error) = app.reload_last_sync() {
                return iced::Task::future(utils::async_error_popup(&utils::error_chain_string(
                    error,
                )))
                .discard();
            }
        }
    }
    iced::Task::none()
}
