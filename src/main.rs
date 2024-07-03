use anyhow::Result;
use frost_byte::gui::{App, WalletMessage};
use frost_byte::spawner::LocalSpawner;
use frost_byte::tasks::Task;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::mpsc;

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

        // Run the GUI
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(
            "Frost byte",
            native_options,
            Box::new(move |cc| Ok(Box::new(App::new(cc, spawner, tx)))),
        )
        .expect("Failed to run app");

        Ok(())
    })
}
