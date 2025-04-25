use anyhow::{Context, Result};
use iced::widget::{self, button, column, row};
use iced::{Element, Length, Task};
use std::path::PathBuf;
use std::sync::Arc;
use utils::async_error_popup;

use crate::db;
use crate::syncing::{self, sync};
use crate::update;

mod lang;
pub mod utils;
mod views;

struct App {
    lang: lang::Lang,
    db: db::AppSettings,
    syncer_state: Option<sync::State>,
    last_sync: Option<syncing::LastSync>,
    currently_syncing: bool,
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
    UpdateApplication,
    Error(Arc<anyhow::Error>),
}

impl App {
    fn new(db: crate::db::AppSettings) -> (Self, Task<Message>) {
        let lang = match match db.get_setting("Lang") {
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

        let last_sync = if let Ok(Some(target_path)) = db.get_setting("target_path") {
            match syncing::get_last_sync(target_path.into())
                .context("error while loading last sync state")
            {
                Err(error) => {
                    let error_message = utils::error_chain_string(error);
                    utils::error_popup(&error_message);
                    panic!("{}", error_message);
                }
                Ok(last_sync) => last_sync,
            }
        } else {
            None
        };

        (
            App {
                lang,
                last_sync,
                db,
                currently_syncing: false,
                syncer_state: None,
            },
            Task::none(),
        )
    }

    fn view(&self) -> Element<'_, Message> {
        let mut root_col = column![
            widget::Container::new(
                row![
                    widget::text(self_update::cargo_crate_version!()),
                    button("Language")
                        .on_press(Message::SwitchLanguage)
                        .style(gray_button),
                    button("Update")
                        .on_press(Message::UpdateApplication)
                        .style(gray_button),
                ]
                .align_y(iced::Alignment::Center)
                .spacing(5)
            )
            .align_x(iced::alignment::Horizontal::Right)
            .width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .padding(10),
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
                            .style(|_, _| widget::svg::Style {
                                color: Some(iced::Color::WHITE)
                            })
                            .width(iced::Length::Shrink)
                        ]
                        .align_y(iced::Alignment::Center)
                        .spacing(10)
                    )
                    .align_x(iced::alignment::Horizontal::Center)
                    .width(Length::Fill)
                )
                .on_press_maybe({
                    if self.currently_syncing {
                        None
                    } else {
                        Some(Message::StartSync)
                    }
                })
                .style(|theme, status| {
                    let mut style = widget::button::primary(theme, status);
                    if status == widget::button::Status::Active {
                        style.background =
                            Some(iced::Background::Color(iced::Color::from_rgb8(50, 200, 50)))
                    } else {
                        style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
                            150, 200, 150,
                        )))
                    }
                    style.border.radius = iced::border::Radius::new(20.0);
                    style.shadow = iced::Shadow {
                        color: iced::Color::from_rgb8(0, 0, 0),
                        offset: iced::Vector::new(0.0, 0.0),
                        blur_radius: 8.0,
                    };
                    style
                })
                .padding(15)
                .width(Length::Fill),
            ]
            .height(Length::FillPortion(20))
            .width(Length::Fill)
            .spacing(10)
            .padding(iced::Padding::from(10.0)),
        ];

        if self.currently_syncing {
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
                .align_x(iced::Alignment::Center),
            )
        }

        root_col.into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(err) => {
                return iced::Task::future(async_error_popup(&utils::error_chain_string(
                    Arc::into_inner(err).unwrap(),
                )))
                .discard();
            }
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
                return views::target::update(self, view_message).map(Message::TargetView);
            }
            Message::SourceView(view_message) => {
                return views::source::update(self, view_message).map(Message::SourceView);
            }
            Message::StartSync => {
                // check if target is set
                let target = match match self.db.get_setting("target_path") {
                    Ok(value) => value,
                    Err(error) => {
                        return Task::future(utils::async_error_popup(&utils::error_chain_string(
                            error,
                        )))
                        .discard();
                    }
                } {
                    None => {
                        return Task::future(utils::async_error_popup(
                            &lang::target_does_not_exist_error(&self.lang),
                        ))
                        .discard();
                    }
                    Some(target_string) => PathBuf::from(target_string),
                };

                // check if sources are available
                let sources = self.db.get_sources().unwrap();
                if sources.is_empty() {
                    return Task::future(utils::async_error_popup(
                        &lang::sources_does_not_exist_error(&self.lang),
                    ))
                    .discard();
                }

                // check if a syncer is already running
                if !self.currently_syncing {
                    // create and set syncer
                    self.currently_syncing = true;
                    return create_sync_task(match sync::Syncer::new(sources, target) {
                        Ok(syncer) => syncer,
                        Err(error) => {
                            return sync_invalid_parameters_popup(&self.lang, error);
                        }
                    });
                }
            }
            Message::FinishedSync => {
                self.currently_syncing = false;
                self.syncer_state = None;
                if let Err(error) = self.reload_last_sync() {
                    return Task::done(Message::Error(error.into()));
                }
            }
            Message::SyncUpdate(state) => self.syncer_state = Some(state),
            Message::UpdateLastSync => {
                if let Err(error) = self.reload_last_sync() {
                    return Task::done(Message::Error(error.into()));
                }
            }
            Message::UpdateApplication => return self.update_application(),
        }
        Task::none()
    }

    fn is_currently_syncing(&self) -> bool {
        self.currently_syncing
    }

    fn update_application(&self) -> iced::Task<Message> {
        let result = update::update();
        match result {
            Ok(status) => iced::Task::future(
                rfd::AsyncMessageDialog::new()
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_title("Updated")
                    .set_description(lang::app_update_finished_description(
                        &self.lang,
                        status.version(),
                        status.uptodate(),
                    ))
                    .show(),
            )
            .discard(),
            Err(error) => {
                iced::Task::future(utils::async_error_popup(&utils::error_chain_string(error)))
                    .discard()
            }
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

fn create_sync_task(mut syncer: sync::Syncer) -> Task<Message> {
    Task::run(
        iced::stream::channel(100, |mut output| async move {
            use iced::futures::sink::SinkExt;

            if let Err(error) = syncer.prepare().await {
                output.send(Message::Error(error.into())).await.unwrap();
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
                            output.send(Message::Error(err.into())).await.unwrap();
                            break;
                        }
                    }
                }
            }

            output.send(Message::FinishedSync).await.unwrap();

            loop {
                tokio::task::yield_now().await;
            }
        }),
        |x| x,
    )
}

fn sync_invalid_parameters_popup(
    lang: &lang::Lang,
    error: sync::InvalidSyncerParameters,
) -> iced::Task<Message> {
    match error {
        sync::InvalidSyncerParameters::SourceDoesNotExist(not_existing_source) => {
            iced::Task::future(utils::async_error_popup(
                &lang::source_does_not_exist_error(lang, &not_existing_source),
            ))
            .discard()
        }
        sync::InvalidSyncerParameters::SourceInTarget(source) => iced::Task::future(
            utils::async_error_popup(&lang::source_in_target_error(lang, &source)),
        )
        .discard(),
        sync::InvalidSyncerParameters::TargetInSource(source) => iced::Task::future(
            utils::async_error_popup(&lang::target_in_source_error(lang, &source)),
        )
        .discard(),
    }
}

pub fn run(db: db::AppSettings) {
    iced::application("File Sync RS", App::update, App::view)
        .subscription(|_| {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::UpdateLastSync)
        })
        .theme(|_| iced::Theme::Light)
        .run_with(move || App::new(db))
        .unwrap();
}

fn gray_button(theme: &iced::Theme, status: widget::button::Status) -> widget::button::Style {
    let mut style = widget::button::primary(theme, status);
    if status == widget::button::Status::Active {
        style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
            207, 207, 207,
        )));
    } else {
        style.background = Some(iced::Background::Color(iced::Color::from_rgb8(
            227, 227, 227,
        )));
    }
    style.border.radius = iced::border::Radius::new(10.0);
    style
}
