use iced::settings::Settings;
use iced::widget::{self, button, column, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Theme};
use std::path::PathBuf;

use crate::db;
use crate::sync;

mod lang;
mod style;

struct Flags {
    db: db::AppSettings,
}

struct App {
    lang: lang::Lang,
    db: db::AppSettings,
    syncer: Option<sync::Syncer>,
    syncer_state: Option<sync::State>,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchLanguage,
    AddFile,
    AddDirectory,
    DeleteSource(PathBuf),
    ChangeTarget,
    StartSync,
    FinishedSync,
    SyncUpdate(sync::State),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                lang: match flags.db.get_setting("Lang").unwrap() {
                    Some(lang_str) => lang::Lang::from(lang_str.as_str()),
                    _ => lang::Lang::English,
                },
                db: flags.db,
                syncer: None,
                syncer_state: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("File Sync RS")
    }

    fn view(&self) -> Element<'_, Message> {
        let mut root_col = column![
            row![button("Language").on_press(Message::SwitchLanguage)].height(Length::Shrink),
            row![
                column![
                    button(lang::add_file(&self.lang)).on_press_maybe({
                        if self.is_currently_syncing() {
                            None
                        } else {
                            Some(Message::AddFile)
                        }
                    }),
                    button(lang::add_directory(&self.lang)).on_press_maybe({
                        if self.is_currently_syncing() {
                            None
                        } else {
                            Some(Message::AddDirectory)
                        }
                    }),
                    widget::Container::new(
                        scrollable(column![self.generate_source_list()]).width(Length::Fill)
                    )
                    .center_y()
                    .height(Length::Fill),
                ]
                .height(Length::Fill)
                .width(Length::FillPortion(1)),
                widget::Container::new(button(lang::start_sync(&self.lang)).on_press_maybe({
                    if self.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::StartSync)
                    }
                }),)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .center_y()
                .center_x(),
                widget::Container::new(self.generate_target_column())
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .center_y()
                    .center_x()
            ]
            .height(Length::FillPortion(20)),
        ];

        if let Some(state) = &self.syncer_state {
            root_col = root_col.push(
                widget::progress_bar(0.0..=state.total_todo() as f32, state.done() as f32)
                    .height(Length::Fixed(10.0)),
            );
        } else if self.syncer.is_some() {
            root_col = root_col
                .push(widget::progress_bar(0_f32..=1_f32, 0_f32).height(Length::Fixed(10.0)))
        }

        root_col.into()
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::SwitchLanguage => {
                let new_lang = match self.lang {
                    lang::Lang::English => lang::Lang::German,
                    lang::Lang::German => lang::Lang::English,
                };
                self.db
                    .set_setting("Lang", String::from(&new_lang).as_str())
                    .unwrap();
                self.lang = new_lang;
            }
            Message::AddFile => {
                self.add_files();
            }
            Message::AddDirectory => {
                self.add_dirs();
            }
            Message::DeleteSource(path) => {
                self.db.remove_source(path).unwrap();
            }
            Message::ChangeTarget => {
                self.change_target();
            }
            Message::StartSync => {
                // check if target is set
                let target = match self.db.get_setting("target_path").unwrap() {
                    None => {
                        rfd::MessageDialog::new()
                            .set_buttons(rfd::MessageButtons::Ok)
                            .set_title("Error")
                            .set_description(lang::target_does_not_exist_error(&self.lang))
                            .show();
                        return Command::none();
                    }
                    Some(target_string) => PathBuf::from(target_string),
                };

                // check if sources are available
                let sources = self.db.get_sources().unwrap();
                if sources.is_empty() {
                    rfd::MessageDialog::new()
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("Error")
                        .set_description(lang::sources_does_not_exist_error(&self.lang))
                        .show();
                    return Command::none();
                }

                // check if a syncer is already running
                if self.syncer.is_none() {
                    // create and set syncer
                    self.syncer = Some(sync::Syncer::new(sources, target))
                }
            }
            Message::FinishedSync => {
                self.syncer = None;
                self.syncer_state = None;
            }
            Message::SyncUpdate(state) => self.syncer_state = Some(state),
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        if self.syncer.is_some() {
            struct Worker;
            let syncer = self.syncer.clone().unwrap();
            iced::subscription::channel(
                std::any::TypeId::of::<Worker>(),
                100,
                |mut output| async move {
                    use iced::futures::sink::SinkExt;

                    tokio::task::spawn_blocking(move || {
                        let runtime = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();
                        for state in syncer {
                            runtime
                                .block_on(output.send(Message::SyncUpdate(state)))
                                .unwrap();
                        }
                        runtime
                            .block_on(output.send(Message::FinishedSync))
                            .unwrap();
                    })
                    .await
                    .unwrap();

                    loop {
                        tokio::task::yield_now().await;
                    }
                },
            )
        } else {
            iced::Subscription::none()
        }
    }
}

impl App {
    fn is_currently_syncing(&self) -> bool {
        self.syncer.is_some()
    }

    fn add_files(&self) {
        if let Some(paths) = rfd::FileDialog::new().pick_files() {
            self.add_source(paths);
        }
    }

    fn add_dirs(&self) {
        if let Some(paths) = rfd::FileDialog::new().pick_folders() {
            self.add_source(paths);
        }
    }

    fn add_source(&self, paths: Vec<PathBuf>) {
        let existing_paths = self.db.get_sources().unwrap();
        'path_loop: for path in paths {
            // check if exact path already exists
            if existing_paths.contains(&path) {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_title("Error")
                    .set_description(lang::source_exists_error(&self.lang, path))
                    .show();
                continue;
            }
            // check if paths overlap
            for existing_path in &existing_paths {
                if existing_path.starts_with(&path) || path.starts_with(existing_path) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("Error")
                        .set_description(lang::sources_overlap_error(
                            &self.lang,
                            &path,
                            existing_path,
                        ))
                        .show();
                    continue 'path_loop;
                }
            }
            // add source
            self.db.add_source(path).unwrap();
        }
    }

    fn change_target(&self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.db
                .set_setting("target_path", path.to_str().unwrap())
                .unwrap();
        }
    }

    fn generate_source_list(&self) -> Element<'_, Message> {
        let mut col = Column::new();
        for path in self.db.get_sources().unwrap() {
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
                        widget::svg::Svg::new(widget::svg::Handle::from_path(
                            "src/gui/assets/trash-fill.svg"
                        ))
                        .style(style::SvgStyleSheet::new(255, 255, 255).into())
                    )
                    .on_press_maybe({
                        if self.is_currently_syncing() {
                            None
                        } else {
                            Some(Message::DeleteSource(path))
                        }
                    })
                    .width(Length::Shrink)
                    .style(
                        style::ButtonStyleSheet::new()
                            .set_background(
                                iced::Color::from_rgb8(230, 30, 30),
                                iced::Color::from_rgb8(230, 100, 100)
                            )
                            .set_border_radius(iced::BorderRadius::from(30.0))
                            .into()
                    )
                ]
                .align_items(iced::Alignment::Center),
            )
        }

        col.into()
    }

    fn generate_target_column(&self) -> Element<'_, Message> {
        let mut col = Column::new().align_items(iced::Alignment::Center);

        col = col.push(
            button(lang::set_target(&self.lang))
                .on_press_maybe({
                    if self.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::ChangeTarget)
                    }
                })
                .style(
                    style::ButtonStyleSheet::new()
                        .set_background(
                            iced::Color::from_rgb8(50, 200, 50),
                            iced::Color::from_rgb8(150, 200, 150),
                        )
                        .into(),
                ),
        );

        if let Some(target) = self.db.get_setting("target_path").unwrap() {
            col = col.push(text(target));
        }

        col.into()
    }
}

pub fn run(db: db::AppSettings) {
    App::run(Settings::with_flags(Flags { db })).unwrap();
}
