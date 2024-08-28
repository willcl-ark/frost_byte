use anyhow::Result;
use clap::Parser;
use frost_byte::gui::{App, WalletMessage};
use frost_byte::spawner::LocalSpawner;
use frost_byte::tasks::Task;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::panic;
use std::path::PathBuf;
use std::process;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to IPC socket
    #[arg(long, conflicts_with = "spawn")]
    socket: Option<PathBuf>,

    /// Auto spawn a bitcoin-node binary at this path
    #[arg(long, conflicts_with = "socket")]
    spawn: Option<String>,
}

fn main() -> Result<()> {
    let spawned_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    // Set up a custom panic hook to alert that bitcoin-node will be left running
    panic::set_hook(Box::new(move |panic_info| {
        eprintln!("Panic occurred: {:?}", panic_info);
        eprintln!("`bitcoin-node` process will be left running and should be terminated manually");
        eprintln!("hint: use `bitcoin-cli [-network] stop` or `pkill bitcoin-node`");
        process::exit(1);
    }));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let result = rt.block_on(async {
        let args = Args::parse();

        let socket_path = match (&args.socket, &args.spawn) {
            (Some(socket), None) => socket.clone(),
            (None, Some(autospawn_cmd)) => {
                let temp_path = random_temp_path();

                let child = Command::new(autospawn_cmd)
                    .arg(format!("-ipcbind=unix://{}", temp_path.to_str().unwrap()))
                    .arg("-debug=ipc")
                    .arg("-regtest")
                    .arg("-daemon=0")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;

                println!("Spawned process: {:?}", child);
                println!("Temporary file created at: {}", temp_path.display());

                // Save the child process
                *spawned_process.lock().unwrap() = Some(child);

                // Wait for the socket to be created
                for _ in 0..30 {
                    // Try for 30 seconds
                    if temp_path.exists() {
                        break;
                    }
                    sleep(Duration::from_secs(1)).await;
                }
                temp_path
            }
            (None, None) => {
                // Neither --socket nor --spawn provided, use default
                let default_path = default_socket_path();
                println!("Using default socket path: {}", default_path.display());
                default_path
            }
            _ => unreachable!("Clap arg parsing gone wrong!"),
        };

        let spawner = LocalSpawner::new();

        // Setup initial connection
        let (send, response) = tokio::sync::oneshot::channel();
        spawner.spawn(Task::SetupConnection(socket_path, send));
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
                        // NodeClient
                        let (send, response) = tokio::sync::oneshot::channel();
                        spawner_clone.spawn(Task::SetupNodeClient(send));
                        match response.await {
                            Ok(Ok(())) => println!("NodeClient setup successfully"),
                            Ok(Err(e)) => println!("Error occurred: {}", e),
                            Err(_) => println!("The sender dropped"),
                        }

                        // WalletLoaderClient
                        // This fails, and I don't know why!
                        let (send, response) = tokio::sync::oneshot::channel();
                        spawner_clone.spawn(Task::SetupWalletLoaderClient(send));
                        match response.await {
                            Ok(Ok(())) => println!("WalletLoaderClient setup successfully"),
                            Ok(Err(e)) => println!("Error occurred: {}", e),
                            Err(_) => println!("The sender dropped"),
                        }

                        // CreateNewWallet
                        let (send, response) = tokio::sync::oneshot::channel();
                        spawner_clone.spawn(Task::CreateNewWallet(send));
                        match response.await {
                            Ok(Ok(())) => println!("Created new wallet setup successfully"),
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
    });

    // Handle clean exit
    if let Some(mut child) = spawned_process.lock().unwrap().take() {
        terminate_process(&mut child);
    }

    result
}

fn default_socket_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".bitcoin")
        .join("sockets")
        .join("node.sock")
}

fn random_temp_path() -> PathBuf {
    let mut rng = thread_rng();
    let random_string: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
    env::temp_dir().join(format!("frost_byte_{}", random_string))
}

fn terminate_process(child: &mut Child) {
    let pid = Pid::from_raw(child.id() as i32);
    eprintln!("Attempting graceful shutdown of spawned process...");
    if let Err(e) = kill(pid, Signal::SIGTERM) {
        eprintln!("Failed to send SIGTERM: {}", e);
        eprintln!("Attempting forceful termination...");
        if let Err(e) = child.kill() {
            eprintln!("Failed to kill spawned process: {}", e);
        }
    }
}
