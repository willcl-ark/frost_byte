use crate::echo_capnp;
use crate::init_capnp;
use crate::proxy_capnp;

// Create Echo client
pub(crate) async fn create_echo_client(
    init_client: &init_capnp::init::Client,
    thread_client: &proxy_capnp::thread::Client,
) -> Result<echo_capnp::echo::Client, Box<dyn std::error::Error>> {
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

    Ok(echo_client_response.get()?.get_result()?)
}

// Send Echo request
pub(crate) async fn send_echo_request(
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
