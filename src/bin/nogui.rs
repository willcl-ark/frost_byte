use anyhow::Result;
use frost_byte::spawner::LocalSpawner;
use frost_byte::tasks::Task;
use std::path::PathBuf;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
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

    // Echo Client setup
    let (send, response) = tokio::sync::oneshot::channel();
    spawner.spawn(Task::SetupEchoClient(send));
    match response.await {
        Ok(Ok(())) => println!("Echo client setup successfully"),
        Ok(Err(e)) => println!("Error occurred: {}", e),
        Err(_) => println!("The sender dropped"),
    }

    Ok(())
}
