use bitcoin_ipc::echo_capnp;
use bitcoin_ipc::init_capnp;
use bitcoin_ipc::proxy_capnp;

use crate::proxy_types::ThreadSafeEchoClient;

// Create Echo client
pub async fn create_echo_client(
    init_client: &init_capnp::init::Client,
    thread_client: &proxy_capnp::thread::Client,
) -> Result<ThreadSafeEchoClient, Box<dyn std::error::Error + Send + Sync>> {
    let mut make_echo_request = init_client.make_echo_request();
    make_echo_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());

    let echo_client_response = make_echo_request.send().promise.await?;
    println!(
        "received echo_client response: {:?}",
        echo_client_response.get()?
    );

    Ok(ThreadSafeEchoClient::new(
        echo_client_response.get()?.get_result()?,
    ))
}

// Send Echo request
pub async fn send_echo_request(
    echo_client: &echo_capnp::echo::Client,
    thread_client: &proxy_capnp::thread::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut new_echo_request = echo_client.echo_request();
    new_echo_request
        .get()
        .get_context()?
        .set_thread(thread_client.clone());
    new_echo_request.get().set_echo("Hello, world!");
    let new_echo = new_echo_request.send().promise.await?;
    println!("received echo response: {:?}", new_echo.get()?);
    Ok(())
}
