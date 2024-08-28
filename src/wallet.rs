use bitcoin_ipc::proxy_capnp;
use bitcoin_ipc::wallet_capnp;

// Manage wallets using the Wallet Loader client
pub async fn create_new_wallet(
    wallet_loader_client: &wallet_capnp::wallet_loader::Client,
    thread_client: &proxy_capnp::thread::Client,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut create_wallet_request = wallet_loader_client.create_wallet_request();
    create_wallet_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());
    create_wallet_request.get().set_name(name);
    create_wallet_request.send().promise.await?;

    let mut list_wallet_dir_request = wallet_loader_client.list_wallet_dir_request();
    list_wallet_dir_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());
    let list_wallet_dir_response = list_wallet_dir_request.send().promise.await?;
    println!(
        "got a list wallet dir response: {:?}",
        list_wallet_dir_response.get()?
    );

    Ok(())
}
