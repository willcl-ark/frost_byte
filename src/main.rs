use std::path::Path;

pub mod chain_capnp;
pub mod common_capnp;
pub mod echo_capnp;
pub mod handler_capnp;
pub mod init_capnp;
pub mod node_capnp;
pub mod proxy_capnp;
pub mod wallet_capnp;

pub mod chain;
pub mod echo;
pub mod init;
pub mod node;
pub mod wallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            // Process args
            let args: Vec<String> = ::std::env::args().collect();
            if args.len() != 2 {
                println!("Usage:\n\t{} [path]", args[0]);
                return Ok(());
            }
            let path = Path::new(&args[1]);

            // Set up connection and get init client and thread client
            let (init_client, thread_client) = init::setup_connection(path).await?;

            // Create and use Echo client
            let echo_client = echo::create_echo_client(&init_client, &thread_client).await?;
            echo::send_echo_request(&echo_client, &thread_client).await?;

            // Create and use Chain client
            let chain_client = chain::create_chain_client(&init_client, &thread_client).await?;
            chain::query_chain_height(&chain_client, &thread_client).await?;

            // Create and use Node client
            let node_client = node::create_node_client(&init_client, &thread_client).await?;

            // Create and use Wallet Loader client from the node
            let wallet_loader_client =
                wallet::create_wallet_loader_client(&node_client, &thread_client).await?;

            // Do something with the wallet
            wallet::create_new_wallet(&wallet_loader_client, &thread_client).await?;
            Ok(())
        })
        .await
}
