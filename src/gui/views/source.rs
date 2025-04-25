use std::path::PathBuf;

use iced::widget::{self, Column, button, column, row, scrollable, text};
use iced::{Element, Length};

use super::super::{App, lang, utils};

#[derive(Debug, Clone)]
pub(in super::super) enum Message {
    AddFile,
    AddDirectory,
    DeleteSource(PathBuf),
}

pub(in super::super) fn view(app: &App) -> Element<'_, Message> {
    widget::Container::new(
        column![
            row![
                button(
                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                        std::borrow::Cow::from(
                            &include_bytes!("../assets/file-earmark-arrow-down.svg")[..]
                        )
                    ))
                    .style(|_, _| widget::svg::Style {
                        color: Some(iced::Color::WHITE)
                    })
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::AddFile)
                    }
                })
                .style(button_style),
                text(lang::source_block_label(&app.lang)),
                button(
                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                        std::borrow::Cow::from(&include_bytes!("../assets/folder-plus.svg")[..])
                    ))
                    .style(|_, _| widget::svg::Style {
                        color: Some(iced::Color::WHITE)
                    })
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::AddDirectory)
                    }
                })
                .style(button_style),
            ]
            .spacing(10),
            widget::Container::new(
                scrollable(column![generate_source_list(app)]).width(Length::Fill)
            )
            .align_x(iced::Center),
        ]
        .width(Length::Fill)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .padding(iced::Padding::from(10.0))
    .style(|_| widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb8(183, 79, 79))),
        border: iced::Border::default().rounded(20.0),
        shadow: iced::Shadow {
            color: iced::Color::from_rgb8(0, 0, 0),
            offset: iced::Vector::new(0.0, 0.0),
            blur_radius: 8.0,
        },
        text_color: None,
    })
    .into()
}

pub(in super::super) fn update(app: &mut App, message: Message) -> iced::Task<Message> {
    match message {
        Message::AddFile => {
            if let Err(error) = add_files(app) {
                return iced::Task::future(utils::async_error_popup(&utils::error_chain_string(error)))
                    .discard();
            }
        }
        Message::AddDirectory => {
            if let Err(error) = add_dirs(app) {
                return iced::Task::future(utils::async_error_popup(&utils::error_chain_string(error)))
                    .discard();
            }
        }
        Message::DeleteSource(path) => {
            if let Err(error) = app.db.remove_source(path) {
                return iced::Task::future(utils::async_error_popup(&utils::error_chain_string(error)))
                    .discard();
            }
        }
    }
    iced::Task::none()
}

fn generate_source_list(app: &App) -> Element<'_, Message> {
    let mut col = Column::new();
    let paths = app.db.get_sources().unwrap(); // FIXME: this unwrap should not be here
    for path in paths {
        col = col.push(
            row![
                scrollable(
                    widget::container::Container::new(text(path.to_str().unwrap().to_string()))
                        .padding(iced::Padding::from(10))
                )
                .direction(widget::scrollable::Direction::Horizontal(
                    widget::scrollable::Scrollbar::new()
                ))
                .width(Length::FillPortion(5)),
                widget::Space::with_width(10),
                button(
                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                        std::borrow::Cow::from(&include_bytes!("../assets/trash-fill.svg")[..])
                    ))
                    .style(|_, _| widget::svg::Style {
                        color: Some(iced::Color::WHITE)
                    })
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::DeleteSource(path))
                    }
                })
                .style(button_style)
            ]
            .align_y(iced::Alignment::Center),
        )
    }

    col.into()
}

fn button_style(theme: &iced::Theme, status: widget::button::Status) -> widget::button::Style {
    let mut style = widget::button::primary(theme, status);
    if status == widget::button::Status::Active {
        style.background = Some(iced::Background::Color(iced::Color::from_rgb8(161, 59, 59)))
    } else {
        style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
            196, 107, 107,
        )))
    }
    style
}

fn add_files(app: &App) -> anyhow::Result<()> {
    if let Some(paths) = rfd::FileDialog::new().pick_files() {
        add_source(app, paths)?;
    }
    Ok(())
}

fn add_dirs(app: &App) -> anyhow::Result<()> {
    if let Some(paths) = rfd::FileDialog::new().pick_folders() {
        add_source(app, paths)?;
    }
    Ok(())
}

fn add_source(app: &App, paths: Vec<PathBuf>) -> anyhow::Result<()> {
    let existing_paths = app.db.get_sources()?;
    for path in paths {
        // check if exact path already exists
        if existing_paths.contains(&path) {
            anyhow::bail!(lang::source_exists_error(&app.lang, path));
        }
        // check if paths overlap
        for existing_path in &existing_paths {
            if existing_path.starts_with(&path) || path.starts_with(existing_path) {
                anyhow::bail!(lang::sources_overlap_error(&app.lang, &path, existing_path,));
            }
        }
        // add source
        app.db.add_source(path)?
    }
    Ok(())
}
