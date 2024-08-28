use crate::clients::Clients;
use crate::echo::create_echo_client;
use crate::wallet::create_new_wallet;
use anyhow::{anyhow, Result};
use bitcoin_ipc::init::setup_connection;
use bitcoin_ipc::node::create_node_client;
use bitcoin_ipc::wallet::create_wallet_loader_client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

pub enum Task {
    SetupConnection(PathBuf, oneshot::Sender<Result<()>>),
    SetupEchoClient(oneshot::Sender<Result<()>>),
    SendEchoRequest(oneshot::Sender<Result<()>>),
    SetupChainClient(oneshot::Sender<Result<()>>),
    SetupNodeClient(oneshot::Sender<Result<()>>),
    SetupWalletLoaderClient(oneshot::Sender<Result<()>>),
    CreateNewWallet(oneshot::Sender<Result<()>>),
}

pub async fn run_task(task: Task, shared_state: Arc<Clients>) -> Result<()> {
    match task {
        Task::SetupConnection(path, response) => {
            println!("Setting up connection with path: {:?}", path);
            let (init_client, thread_client) = match setup_connection(path.as_ref()).await {
                Ok(clients) => {
                    println!("Connection setup successful");
                    clients
                }
                Err(e) => {
                    let err = anyhow!("Failed to setup connection: {}", e);
                    eprintln!("Error: {}", err);
                    response
                        .send(Err(err))
                        .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                    return Err(anyhow!("Failed to setup connection"));
                }
            };

            {
                let mut init_lock = shared_state.init_client.write().unwrap();
                let mut thread_lock = shared_state.thread_client.write().unwrap();
                *init_lock = Some(init_client);
                *thread_lock = Some(thread_client);
                println!("Clients stored in shared state");
            }

            response
                .send(Ok(()))
                .unwrap_or_else(|_| eprintln!("Failed to send success response"));
        }
        Task::SetupEchoClient(response) => {
            println!("Setting up Echo client");
            let init_client = shared_state.init_client.read().unwrap();
            let thread_client = shared_state.thread_client.read().unwrap();

            if let (Some(ref init), Some(ref thread)) = (&*init_client, &*thread_client) {
                println!("Init and Thread clients available");
                let echo_client = match create_echo_client(init, thread).await {
                    Ok(client) => {
                        println!("Echo client created successfully");
                        client
                    }
                    Err(e) => {
                        let err = anyhow!("Failed to create echo client: {}", e);
                        eprintln!("Error: {}", err);
                        response
                            .send(Err(err))
                            .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                        return Err(anyhow!("Failed to create echo client"));
                    }
                };

                {
                    let mut echo_lock = shared_state.echo_client.write().unwrap();
                    *echo_lock = Some(echo_client);
                    println!("Echo client stored in shared state");
                }

                response
                    .send(Ok(()))
                    .unwrap_or_else(|_| eprintln!("Failed to send success response"));
            } else {
                let err = anyhow!("Clients are not initialized");
                eprintln!("Error: {}", err);
                response
                    .send(Err(err))
                    .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                return Err(anyhow!("Clients are not initialized"));
            }
        }
        Task::SetupNodeClient(response) => {
            println!("Setting up Node client");
            let init_client = shared_state.init_client.read().unwrap();
            let thread_client = shared_state.thread_client.read().unwrap();

            if let (Some(ref init), Some(ref thread)) = (&*init_client, &*thread_client) {
                println!("Init and Thread clients available");
                let node_client = match create_node_client(init, thread).await {
                    Ok(client) => {
                        println!("Node client created successfully");
                        client
                    }
                    Err(e) => {
                        let err = anyhow!("Failed to create node client: {}", e);
                        eprintln!("Error: {}", err);
                        response
                            .send(Err(err))
                            .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                        return Err(anyhow!("Failed to create node client"));
                    }
                };

                {
                    let mut node_lock = shared_state.node_client.write().unwrap();
                    *node_lock = Some(node_client);
                    println!("Node client stored in shared state");
                }

                response
                    .send(Ok(()))
                    .unwrap_or_else(|_| eprintln!("Failed to send success response"));
            } else {
                let err = anyhow!("Clients are not initialized");
                eprintln!("Error: {}", err);
                response
                    .send(Err(err))
                    .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                return Err(anyhow!("Clients are not initialized"));
            }
        }
        Task::SetupWalletLoaderClient(response) => {
            println!("Setting up WalletLoader client");
            let node_client = shared_state.node_client.read().unwrap();
            let thread_client = shared_state.thread_client.read().unwrap();

            if let (Some(ref node), Some(ref thread)) = (&*node_client, &*thread_client) {
                println!("Node and Thread clients available");
                let wallet_client = match create_wallet_loader_client(node, thread).await {
                    Ok(client) => {
                        println!("WalletLoader client created successfully");
                        client
                    }
                    Err(e) => {
                        let err = anyhow!("Failed to create wallet loader client: {}", e);
                        eprintln!("Error: {}", err);
                        response
                            .send(Err(err))
                            .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                        return Err(anyhow!("Failed to create wallet loader client"));
                    }
                };

                {
                    let mut wallet_lock = shared_state.wallet_loader_client.write().unwrap();
                    *wallet_lock = Some(wallet_client);
                    println!("WalletLoader client stored in shared state");
                }

                response
                    .send(Ok(()))
                    .unwrap_or_else(|_| eprintln!("Failed to send success response"));
            } else {
                let err = anyhow!("Clients are not initialized");
                eprintln!("Error: {}", err);
                response
                    .send(Err(err))
                    .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                return Err(anyhow!("Clients are not initialized"));
            }
        }
        Task::CreateNewWallet(response) => {
            println!("Creating new wallet");
            let wallet_loader_client = shared_state.wallet_loader_client.read().unwrap();
            let thread_client = shared_state.thread_client.read().unwrap();
            if let (Some(ref wallet_loader), Some(ref thread)) =
                (&*wallet_loader_client, &*thread_client)
            {
                println!("WalletLoader and Thread clients available");
                match create_new_wallet(wallet_loader, thread, "frost_byte999").await {
                    Ok(_) => println!("New wallet created successfully"),
                    Err(e) => eprintln!("Error creating new wallet: {}", e),
                }
                response
                    .send(Ok(()))
                    .unwrap_or_else(|_| eprintln!("Failed to send success response"));
            } else {
                let err = anyhow!("Clients are not initialized");
                eprintln!("Error: {}", err);
                response
                    .send(Err(err))
                    .unwrap_or_else(|_| eprintln!("Failed to send error response"));
                return Err(anyhow!("Clients are not initialized"));
            }
        }
        _ => {
            eprintln!("Unimplemented task encountered");
            unimplemented!()
        }
    }
    Ok(())
}
