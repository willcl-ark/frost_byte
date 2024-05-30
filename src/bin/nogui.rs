use std::error::Error;
use std::path::Path;

use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::task::LocalSet;

use frost_byte::chain;
use frost_byte::echo;
use frost_byte::init;
use frost_byte::node;
use frost_byte::wallet;

// This struct describes the task you want to spawn. Here we include
// some simple examples. The oneshot channel allows sending a response
// to the spawner.
#[derive(Debug)]
enum Task {
    SetupConnection(mpsc::Sender<Result<(), Box<dyn Error + Send + Sync>>>),
    // CreateNewWallet(String, mpsc::Sender<Result<()>>),
}

#[derive(Clone)]
struct LocalSpawner {
    send: mpsc::UnboundedSender<Task>,
}

impl LocalSpawner {
    pub fn new() -> Self {
        let (send, mut recv) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(new_task) = recv.recv().await {
                    tokio::task::spawn_local(async move {
                        if let Err(e) = run_task(new_task).await {
                            eprintln!("Error running task: {:?}", e);
                        }
                    });
                }
                // If the while loop returns, then all the LocalSpawner
                // objects have been dropped.
            });

            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });

        Self { send }
    }

    pub fn spawn(&self, task: Task) {
        self.send
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }
}

// This task may do !Send stuff.
//
// The Task struct is an enum to support spawning many different kinds
// of operations.
//
// TODO! We need to handle all these unwraps properly
//
async fn run_task(task: Task) -> Result<(), Box<dyn Error + Send + Sync>> {
    match task {
        Task::SetupConnection(response) => {
            // Process args
            let args: Vec<String> = ::std::env::args().collect();
            if args.len() != 2 {
                println!("Usage:\n\t{} [path]", args[0]);
                response.send(Ok(())).await.unwrap();
                return Ok(());
            }
            let path = Path::new(&args[1]);

            // Set up connection and get init client and thread client
            let (init_client, thread_client) = init::setup_connection(path).await.unwrap();

            // Create and use Echo client
            let echo_client = echo::create_echo_client(&init_client, &thread_client)
                .await
                .unwrap();
            echo::send_echo_request(&echo_client, &thread_client)
                .await
                .unwrap();

            // Create and use Chain client
            let chain_client = chain::create_chain_client(&init_client, &thread_client)
                .await
                .unwrap();
            chain::query_chain_height(&chain_client, &thread_client)
                .await
                .unwrap();

            // Create and use Node client
            let node_client = node::create_node_client(&init_client, &thread_client)
                .await
                .unwrap();

            // Create and use Wallet Loader client from the node
            let wallet_loader_client =
                wallet::create_wallet_loader_client(&node_client, &thread_client)
                    .await
                    .unwrap();

            // We ignore failures to send the response.
            // Do something with the wallet
            wallet::create_new_wallet(&wallet_loader_client, &thread_client)
                .await
                .unwrap();
            response.send(Ok(())).await.unwrap();
        } // Task::CreateNewWallet(name, response) => {
          //     // We ignore failures to send the response.
          //     // Do something with the wallet
          //     wallet::create_new_wallet(&wallet_loader_client, &thread_client).await?;
          //     response.send(Ok(()));
          // }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let spawner = LocalSpawner::new();

    let (send, _response) = mpsc::channel(10);
    spawner.spawn(Task::SetupConnection(send.clone()));
    // if response.await.is_ok() {
    //     spawner.spawn(Task::CreateNewWallet("tokio-spawn".to_string(), send));
    //     if response.await.is_ok() {
    //         println!("Created a new wallet!");
    //     }
    // }
    println!("Done!");
}
