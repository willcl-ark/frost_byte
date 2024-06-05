use crate::spawner::LocalSpawner;
use iced::widget::button::State;
use iced::widget::{button, column};
use iced::{executor, Application, Command, Element, Theme};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum WalletMessage {
    CreateNewWallet,
}

pub struct App {
    create_wallet_button: State,
    spawner: LocalSpawner,
    tx: mpsc::UnboundedSender<WalletMessage>,
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

    fn view(&self) -> Element<Self::Message> {
        column![button("Create new wallet").on_press(WalletMessage::CreateNewWallet),].into()
    }
}
