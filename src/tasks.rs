use crate::clients::Clients;
use crate::echo::create_echo_client;
use anyhow::{anyhow, Result};
use bitcoin_ipc::init::setup_connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

pub enum Task {
    SetupConnection(PathBuf, oneshot::Sender<Result<()>>),
    SetupEchoClient(oneshot::Sender<Result<()>>),
    SendEchoRequest(oneshot::Sender<Result<()>>),
    SetupChainClient(oneshot::Sender<Result<()>>),
    SetupNodeClient(oneshot::Sender<Result<()>>),
}

pub async fn run_task(task: Task, shared_state: Arc<Clients>) -> Result<()> {
    match task {
        Task::SetupConnection(path, response) => {
            let (init_client, thread_client) = match setup_connection(path.as_ref()).await {
                Ok(clients) => clients,
                Err(e) => {
                    let err = anyhow!("Failed to setup connection: {}", e);
                    response.send(Err(err)).unwrap_or(());
                    return Err(anyhow!("Failed to setup connection"));
                }
            };

            {
                let mut init_lock = shared_state.init_client.write().unwrap();
                let mut thread_lock = shared_state.thread_client.write().unwrap();
                *init_lock = Some(init_client);
                *thread_lock = Some(thread_client);
            }

            response.send(Ok(())).unwrap_or(());
        }
        Task::SetupEchoClient(response) => {
            let init_client = shared_state.init_client.read().unwrap();
            let thread_client = shared_state.thread_client.read().unwrap();

            // Check if the clients are available
            if let (Some(ref init), Some(ref thread)) = (&*init_client, &*thread_client) {
                let echo_client = match create_echo_client(init, thread).await {
                    Ok(client) => client,
                    Err(e) => {
                        let err = anyhow!("Failed to create echo client: {}", e);
                        response.send(Err(err)).unwrap_or(());
                        return Err(anyhow!("Failed to create echo client"));
                    }
                };

                {
                    let mut echo_lock = shared_state.echo_client.write().unwrap();
                    *echo_lock = Some(echo_client);
                }

                response.send(Ok(())).unwrap_or(());
            } else {
                let err = anyhow!("Clients are not initialized");
                response.send(Err(err)).unwrap_or(());
                return Err(anyhow!("Clients are not initialized"));
            }
        }
        _ => unimplemented!(),
    }
    Ok(())
}
