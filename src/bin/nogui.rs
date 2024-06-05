use anyhow::Result;
use frost_byte::spawner::LocalSpawner;
use frost_byte::tasks::Task;
use iced::widget::button::State;
use iced::widget::{button, column};
use iced::{executor, Application, Command, Element, Settings, Theme};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
enum WalletMessage {
    CreateNewWallet,
}

struct MyApp {
    create_wallet_button: State,
    spawner: LocalSpawner,
    tx: mpsc::UnboundedSender<WalletMessage>,
}

impl Application for MyApp {
    type Executor = executor::Default;
    type Message = WalletMessage;
    type Flags = (LocalSpawner, mpsc::UnboundedSender<WalletMessage>);
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            MyApp {
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

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            println!("Usage:\n\t{} [path]", args[0]);
            return Ok(());
        }
        let path = PathBuf::from_str(&args[1]).unwrap();

        let spawner = LocalSpawner::new();

        // Setup initial connection
        let (send, response) = tokio::sync::oneshot::channel();
        spawner.spawn(Task::SetupConnection(path, send));
        match response.await {
            Ok(Ok(())) => println!("Connection setup successfully"),
            Ok(Err(e)) => println!("Error occurred: {}", e),
            Err(_) => println!("The sender dropped"),
        }

        // Setup communication channel
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Spawn a task to handle messages from the GUI
        let spawner_clone = spawner.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    WalletMessage::CreateNewWallet => {
                        let (send, response) = tokio::sync::oneshot::channel();
                        spawner_clone.spawn(Task::SetupEchoClient(send));
                        match response.await {
                            Ok(Ok(())) => println!("Echo client setup successfully"),
                            Ok(Err(e)) => println!("Error occurred: {}", e),
                            Err(_) => println!("The sender dropped"),
                        }
                    }
                }
            }
        });

        // Run the GUI on the main thread
        iced::Settings::with_flags((spawner.clone(), tx.clone()));
        MyApp::run(Settings::with_flags((spawner, tx))).unwrap();

        Ok(())
    })
}
