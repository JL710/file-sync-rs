use anyhow::{Context, Result};
use iced::settings::Settings;
use iced::widget::{self, button, column, row};
use iced::{executor, Application, Command, Element, Length, Theme};
use std::path::PathBuf;

use crate::db;
use crate::syncing::{self, sync};

mod lang;
mod style;
pub mod utils;
mod views;

struct Flags {
    db: db::AppSettings,
}

struct App {
    lang: lang::Lang,
    db: db::AppSettings,
    syncer: Option<sync::Syncer>,
    syncer_state: Option<sync::State>,
    last_sync: Option<syncing::LastSync>,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchLanguage,
    TargetView(views::target::Message),
    SourceView(views::source::Message),
    StartSync,
    FinishedSync,
    SyncUpdate(sync::State),
    UpdateLastSync,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Flags) -> (Self, Command<Self::Message>) {
        let lang = match match flags.db.get_setting("Lang") {
            Ok(value) => value,
            Err(error) => {
                let error_string = utils::error_chain_string(error);
                utils::error_popup(&error_string);
                panic!("{}", error_string);
            }
        } {
            Some(lang_str) => lang::Lang::from(lang_str.as_str()),
            _ => lang::Lang::English,
        };

        (
            App {
                lang,
                last_sync: if let Ok(Some(target_path)) = flags.db.get_setting("target_path") {
                    match syncing::get_last_sync(target_path.into()) {
                        Err(error) => {
                            let error_message = utils::error_chain_string(error);
                            utils::error_popup(&error_message);
                            panic!("{}", error_message);
                        }
                        Ok(last_sync) => last_sync,
                    }
                } else {
                    None
                },
                db: flags.db,
                syncer: None,
                syncer_state: None,
            },
            iced::command::channel(100, |mut channel| async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    channel
                        .try_send(Message::UpdateLastSync)
                        .expect("Could not send last sync message");
                }
            }),
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
                    .style(iced::theme::Button::custom(
                        style::ButtonStyleSheet::new()
                            .set_border(iced::Border::with_radius(10.0))
                            .set_background(
                                iced::Color::from_rgb8(207, 207, 207),
                                iced::Color::from_rgb8(227, 227, 227)
                            )
                    ))
                    .padding(5)
            )
            .align_x(iced::alignment::Horizontal::Right)
            .width(iced::Length::Fill)
            .padding(iced::Padding::from(10))]
            .height(Length::Shrink),
            column![
                views::source::view(self).map(Message::SourceView),
                views::target::view(self).map(Message::TargetView),
                button(
                    widget::Container::new(
                        widget::row![
                            lang::start_sync(&self.lang),
                            widget::svg::Svg::new(widget::svg::Handle::from_memory(
                                std::borrow::Cow::from(
                                    &include_bytes!("./assets/file-earmark-play.svg")[..]
                                )
                            ))
                            .style(iced::theme::Svg::Custom(Box::new(
                                style::SvgStyleSheet::new(255, 255, 255)
                            )))
                            .width(iced::Length::Shrink)
                        ]
                        .align_items(iced::Alignment::Center)
                        .spacing(10)
                    )
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(Length::Fill)
                )
                .on_press_maybe({
                    if self.is_currently_syncing() {
                        None
                    } else {
                        Some(Message::StartSync)
                    }
                })
                .style(iced::theme::Button::custom(
                    style::ButtonStyleSheet::new()
                        .set_background(
                            iced::Color::from_rgb8(50, 200, 50),
                            iced::Color::from_rgb8(150, 200, 150),
                        )
                        .set_border(iced::Border::with_radius(20.0))
                ),)
                .padding(15)
                .width(Length::Fill),
            ]
            .height(Length::FillPortion(20))
            .width(Length::Fill)
            .spacing(10)
            .padding(iced::Padding::from(10.0)),
        ];

        if self.syncer.is_some() {
            root_col = root_col.push(
                column![
                    widget::text(format!(
                        "Current Files: {}",
                        if let Some(state) = &self.syncer_state {
                            state
                                .current_work()
                                .iter()
                                .map(|path| path.to_str().unwrap())
                                .collect::<Vec<&str>>()
                                .join(", ")
                        } else {
                            String::from("Indexing")
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
                if let Err(error) = self.reload_last_sync() {
                    utils::error_popup(&utils::error_chain_string(error));
                }
            }
            Message::SyncUpdate(state) => self.syncer_state = Some(state),
            Message::UpdateLastSync => {
                if let Err(error) = self.reload_last_sync() {
                    utils::error_popup(&utils::error_chain_string(error));
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let mut subscriptions = Vec::new();

        // syncer subscription
        if self.syncer.is_some() {
            struct Worker;
            let mut syncer = self.syncer.clone().unwrap();
            subscriptions.push(iced::subscription::channel(
                std::any::TypeId::of::<Worker>(),
                100,
                |mut output| async move {
                    use iced::futures::sink::SinkExt;

                    if let Err(error) = syncer.prepare().await {
                        utils::error_popup(&utils::error_chain_string(error));
                    } else {
                        loop {
                            let syncer_result = syncer.async_next().await;
                            match syncer_result {
                                None => {
                                    break;
                                }
                                Some(Ok(state)) => {
                                    output.send(Message::SyncUpdate(state)).await.unwrap();
                                }
                                Some(Err(err)) => {
                                    utils::error_popup(&utils::error_chain_string(err));
                                    break;
                                }
                            }
                        }
                    }

                    output.send(Message::FinishedSync).await.unwrap();

                    loop {
                        tokio::task::yield_now().await;
                    }
                },
            ))
        }

        // return subscriptions
        iced::Subscription::batch(subscriptions)
    }
}

impl App {
    fn is_currently_syncing(&self) -> bool {
        self.syncer.is_some()
    }

    fn start_sync(&mut self) {
        // check if target is set
        let target = match match self.db.get_setting("target_path") {
            Ok(value) => value,
            Err(error) => {
                utils::error_popup(&utils::error_chain_string(error));
                return;
            }
        } {
            None => {
                utils::error_popup(&lang::target_does_not_exist_error(&self.lang));
                return;
            }
            Some(target_string) => PathBuf::from(target_string),
        };

        // check if sources are available
        let sources = self.db.get_sources().unwrap();
        if sources.is_empty() {
            utils::error_popup(&lang::sources_does_not_exist_error(&self.lang));
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

    fn reload_last_sync(&mut self) -> Result<()> {
        self.last_sync = if let Ok(Some(target_path)) = self.db.get_setting("target_path") {
            syncing::get_last_sync(target_path.into()).context("failed to load setting from db")?
        } else {
            None
        };

        Ok(())
    }
}

fn sync_invalid_parameters_popup(lang: &lang::Lang, error: sync::InvalidSyncerParameters) {
    match error {
        sync::InvalidSyncerParameters::SourceDoesNotExist(not_existing_source) => {
            utils::error_popup(&lang::source_does_not_exist_error(
                lang,
                &not_existing_source,
            ));
        }
        sync::InvalidSyncerParameters::SourceInTarget(source) => {
            utils::error_popup(&lang::source_in_target_error(lang, &source));
        }
        sync::InvalidSyncerParameters::TargetInSource(source) => {
            utils::error_popup(&lang::target_in_source_error(lang, &source));
        }
    }
}

pub fn run(db: db::AppSettings) {
    App::run(Settings::with_flags(Flags { db })).unwrap();
}
