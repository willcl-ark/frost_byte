use iced::{
    widget::{button, column},
    Sandbox, Settings,
};

fn main() -> iced::Result {
    MyApp::run(Settings::default())
}

#[derive(Debug, Clone)]
enum WalletMessage {
    CreateNewWallet,
}

struct MyApp;

impl Sandbox for MyApp {
    type Message = WalletMessage;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("My App")
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> iced::Element<Self::Message> {
        column![button("Create new wallet").on_press(WalletMessage::CreateNewWallet),].into()
    }
}
