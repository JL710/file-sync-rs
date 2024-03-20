use std::path::PathBuf;

use iced::widget::{self, button, column, row, scrollable, text, Column};
use iced::{Element, Length};

use super::super::{lang, style, utils, App};

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
                    .style(style::SvgStyleSheet::new(255, 255, 255))
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::AddFile)
                    }
                })
                .style(style::ButtonStyleSheet::new().set_background(
                    iced::Color::from_rgb8(161, 59, 59),
                    iced::Color::from_rgb8(196, 107, 107)
                )),
                text(lang::source_block_label(&app.lang)),
                button(
                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                        std::borrow::Cow::from(&include_bytes!("../assets/folder-plus.svg")[..])
                    ))
                    .style(style::SvgStyleSheet::new(255, 255, 255))
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::AddDirectory)
                    }
                })
                .style(style::ButtonStyleSheet::new().set_background(
                    iced::Color::from_rgb8(161, 59, 59),
                    iced::Color::from_rgb8(196, 107, 107)
                )),
            ]
            .spacing(10),
            widget::Container::new(
                scrollable(column![generate_source_list(app)]).width(Length::Fill)
            )
            .center_y(),
        ]
        .width(Length::Fill)
        .align_items(iced::Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .height(Length::Fill)
    .padding(iced::Padding::from(10.0))
    .style(
        style::ContainerStyleSheet::new()
            .background(Some(iced::Background::Color(iced::Color::from_rgb8(
                183, 79, 79,
            ))))
            .border_radius(iced::Border::with_radius(20.0))
            .shadow(iced::Shadow {
                color: iced::Color::from_rgb8(0, 0, 0),
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 8.0,
            }),
    )
    .into()
}

pub(in super::super) fn update(app: &mut App, message: Message) {
    match message {
        Message::AddFile => {
            add_files(app);
        }
        Message::AddDirectory => {
            add_dirs(app);
        }
        Message::DeleteSource(path) => {
            if let Err(error) = app.db.remove_source(path) {
                utils::error_popup(&utils::error_chain_string(error));
            }
        }
    }
}

fn generate_source_list(app: &App) -> Element<'_, Message> {
    let mut col = Column::new();
    let paths = match app.db.get_sources() {
        Ok(value) => value,
        Err(error) => {
            utils::error_popup(&utils::error_chain_string(error));
            Vec::new()
        }
    };
    for path in paths {
        col = col.push(
            row![
                scrollable(
                    widget::container::Container::new(text(path.to_str().unwrap()))
                        .padding(iced::Padding::from(10))
                )
                .direction(widget::scrollable::Direction::Horizontal(
                    widget::scrollable::Properties::new()
                ))
                .width(Length::FillPortion(5)),
                widget::Space::with_width(10),
                button(
                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                        std::borrow::Cow::from(&include_bytes!("../assets/trash-fill.svg")[..])
                    ))
                    .style(style::SvgStyleSheet::new(255, 255, 255))
                    .width(Length::Shrink)
                )
                .on_press_maybe({
                    if app.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::DeleteSource(path))
                    }
                })
                .style(
                    style::ButtonStyleSheet::new()
                        .set_background(
                            iced::Color::from_rgb8(161, 59, 59),
                            iced::Color::from_rgb8(196, 107, 107)
                        )
                        .set_border(iced::Border::with_radius(30.0))
                )
            ]
            .align_items(iced::Alignment::Center),
        )
    }

    col.into()
}

fn add_files(app: &App) {
    if let Some(paths) = rfd::FileDialog::new().pick_files() {
        add_source(app, paths);
    }
}

fn add_dirs(app: &App) {
    if let Some(paths) = rfd::FileDialog::new().pick_folders() {
        add_source(app, paths);
    }
}

fn add_source(app: &App, paths: Vec<PathBuf>) {
    let existing_paths = match app.db.get_sources() {
        Ok(value) => value,
        Err(error) => {
            utils::error_popup(&utils::error_chain_string(error));
            return;
        }
    };
    'path_loop: for path in paths {
        // check if exact path already exists
        if existing_paths.contains(&path) {
            utils::error_popup(&lang::source_exists_error(&app.lang, path));
            continue;
        }
        // check if paths overlap
        for existing_path in &existing_paths {
            if existing_path.starts_with(&path) || path.starts_with(existing_path) {
                utils::error_popup(&lang::sources_overlap_error(
                    &app.lang,
                    &path,
                    existing_path,
                ));
                continue 'path_loop;
            }
        }
        // add source
        if let Err(error) = app.db.add_source(path) {
            utils::error_popup(&utils::error_chain_string(error));
        }
    }
}
