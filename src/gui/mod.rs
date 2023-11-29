use iced::settings::Settings;
use iced::widget::{self, button, column, row};
use iced::{executor, Application, Command, Element, Length, Theme};
use std::path::PathBuf;

use crate::db;
use crate::sync;

mod lang;
mod style;
mod views;

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
    TargetView(views::target::Message),
    SourceView(views::source::Message),
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
            row![widget::Container::new(
                button("Language")
                    .on_press(Message::SwitchLanguage)
                    .style(
                        style::ButtonStyleSheet::new()
                            .set_border_radius(10.0)
                            .set_background(
                                iced::Color::from_rgb8(207, 207, 207),
                                iced::Color::from_rgb8(227, 227, 227)
                            )
                            .into()
                    )
                    .padding(5)
            )
            .align_x(iced::alignment::Horizontal::Right)
            .width(iced::Length::Fill)
            .padding(iced::Padding::from(10))]
            .height(Length::Shrink),
            row![
                views::source::view(self).map(Message::SourceView),
                widget::Container::new(
                    button(
                        widget::row![
                            lang::start_sync(&self.lang),
                            widget::svg::Svg::new(widget::svg::Handle::from_memory(
                                std::borrow::Cow::from(
                                    &include_bytes!("./assets/file-earmark-play.svg")[..]
                                )
                            ))
                            .style(style::SvgStyleSheet::new(255, 255, 255).into())
                            .width(iced::Length::Shrink)
                        ]
                        .align_items(iced::Alignment::Center)
                        .spacing(10)
                    )
                    .on_press_maybe({
                        if self.is_currently_syncing() {
                            None
                        } else {
                            Some(Message::StartSync)
                        }
                    })
                    .style(
                        style::ButtonStyleSheet::new()
                            .set_background(
                                iced::Color::from_rgb8(50, 200, 50),
                                iced::Color::from_rgb8(150, 200, 150),
                            )
                            .set_border_radius(20.0)
                            .into(),
                    )
                    .padding(15),
                )
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .center_y()
                .center_x(),
                widget::Container::new(views::target::view(self).map(Message::TargetView))
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .center_y()
                    .center_x()
            ]
            .height(Length::FillPortion(20))
            .padding(iced::Padding::from(10.0)),
        ];

        if self.syncer.is_some() {
            root_col = root_col.push(
                column![
                    widget::text(format!(
                        "Current Files: {}",
                        if let Some(state) = &self.syncer_state {
                            state.current_file().to_str().unwrap()
                        } else {
                            "Indexing"
                        }
                    )),
                    widget::progress_bar(
                        0_f32..=if let Some(state) = &self.syncer_state {
                            state.total()
                        } else {
                            1
                        } as f32,
                        if let Some(state) = &self.syncer_state {
                            state.done()
                        } else {
                            0
                        } as f32,
                    )
                    .height(Length::Fixed(10.0)),
                ]
                .align_items(iced::Alignment::Center),
            )
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
            Message::TargetView(view_message) => {
                views::target::update(self, view_message);
            }
            Message::SourceView(view_message) => {
                views::source::update(self, view_message);
            }
            Message::StartSync => {
                self.start_sync();
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
            let mut syncer = self.syncer.clone().unwrap();
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
                        syncer.resolve();
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

    fn start_sync(&mut self) {
        // check if target is set
        let target = match self.db.get_setting("target_path").unwrap() {
            None => {
                rfd::MessageDialog::new()
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_title("Error")
                    .set_description(lang::target_does_not_exist_error(&self.lang))
                    .show();
                return;
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
            return;
        }

        // check if a syncer is already running
        if self.syncer.is_none() {
            // create and set syncer
            self.syncer = Some(match sync::Syncer::new(sources, target) {
                Ok(syncer) => syncer,
                Err(error) => {
                    sync_invalid_parameters_popup(&self.lang, error);
                    return;
                }
            })
        }
    }
}

fn sync_invalid_parameters_popup(lang: &lang::Lang, error: sync::InvalidSyncerParameters) {
    match error {
        sync::InvalidSyncerParameters::SourceDoesNotExist(not_existing_source) => {
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description(lang::source_does_not_exist_error(
                    lang,
                    &not_existing_source,
                ))
                .show();
        }
        sync::InvalidSyncerParameters::SourceInTarget(source) => {
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description(lang::source_in_target_error(lang, &source))
                .show();
        }
        sync::InvalidSyncerParameters::TargetInSource(source) => {
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_buttons(rfd::MessageButtons::Ok)
                .set_description(lang::target_in_source_error(lang, &source))
                .show();
        }
    }
}

pub fn run(db: db::AppSettings) {
    App::run(Settings::with_flags(Flags { db })).unwrap();
}
