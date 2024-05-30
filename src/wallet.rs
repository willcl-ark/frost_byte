use crate::node_capnp;
use crate::proxy_capnp;
use crate::wallet_capnp;

// Manage wallets using the Node client
pub async fn create_wallet_loader_client(
    node_client: &node_capnp::node::Client,
    thread_client: &proxy_capnp::thread::Client,
) -> Result<wallet_capnp::wallet_loader::Client, Box<dyn std::error::Error>> {
    let mut custom_wallet_loader_request = node_client.custom_wallet_loader_request();
    custom_wallet_loader_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());
    let cwl_response = custom_wallet_loader_request.send().promise.await?;
    println!(
        "received create_wallet_loader_client response: {:?}",
        cwl_response.get()?
    );
    Ok(cwl_response.get()?.get_result()?)
}

// Manage wallets using the Wallet Loader client
pub async fn create_new_wallet(
    wallet_loader_client: &wallet_capnp::wallet_loader::Client,
    thread_client: &proxy_capnp::thread::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut create_wallet_request = wallet_loader_client.create_wallet_request();
    create_wallet_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());
    create_wallet_request.get().set_name("frost_byte");
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
