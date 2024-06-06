use crate::spawner::LocalSpawner;
use iced::widget::button::State;
use iced::widget::{button, column, progress_bar};
use iced::{executor, Application, Command, Element, Theme};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Progress {
    Started,
    Advanced(f32),
    Finished,
    Error,
}

pub enum State {
    Idle,
    Syncing { progress: f32 },
    Finished,
    Errored,
}

#[derive(Debug, Clone)]
pub enum WalletMessage {
    CreateNewWallet,
    SyncProgressed(Progress),
}

#[derive(Debug)]
struct SyncProgressed {
    state: State,
}

pub struct App {
    create_wallet_button: State,
    spawner: LocalSpawner,
    tx: mpsc::UnboundedSender<WalletMessage>,
    state: State,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = WalletMessage;
    type Flags = (LocalSpawner, mpsc::UnboundedSender<WalletMessage>);
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                create_wallet_button: State::new(),
                spawner: flags.0,
                tx: flags.1,
                state: State::Idle,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("My App")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            WalletMessage::CreateNewWallet => {
                let _ = self.tx.send(WalletMessage::CreateNewWallet);
            }
        }
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<Self::Message> {
        match self.state {
            State::Syncing { .. } => {
                GetProgress()
                    .map(Message::DownloadProgressed)
            }
            _ => Subscription::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let current_progress = match &self.state {
            State::Idle { .. } => 0.0,
            State::Syncing { progress } => *progress,
            State::Finished { .. } => 100.0,
            State::Errored { .. } => 0.0,
        };

        let progress_bar = progress_bar(0.0..=100.0, current_progress);
        let control = button("Create new wallet").on_press(WalletMessage::CreateNewWallet)
        Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center)
            .push(progress_bar)
            .push(control)
            .into()
    }
}
