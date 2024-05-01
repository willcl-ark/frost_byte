use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use tokio::net::UnixStream;
use tokio_util::compat::*;

pub mod chain_capnp;
pub mod common_capnp;
pub mod handler_capnp;
pub mod proxy_capnp;
pub mod wallet_capnp;
pub mod init_capnp;
pub mod echo_capnp;
pub mod node_capnp;

use crate::init_capnp::init::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            let path = std::path::Path::new("/root/.bitcoin/sockets/node.sock");
            let stream = UnixStream::connect(path).await?;
            let (reader, writer) = stream.into_split();

            let reader_compat = reader.compat();
            let writer_compat = writer.compat_write();

            let rpc_network = Box::new(twoparty::VatNetwork::new(
                reader_compat,
                writer_compat,
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));

            let mut rpc_system = RpcSystem::new(rpc_network, None);
            let frost_byte: Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
            tokio::task::spawn_local(rpc_system);

            println!("make echo request");
            let request = frost_byte.make_echo_request();
            println!("send echo request");
            let reply = request.send().promise.await?;
            println!("get echo request");
            let echo = reply.get()?;
            let client = echo.get_result()?;
            let mut echo_request = client.echo_request();
            let msg = String::from("Hi bill");
            echo_request.get().set_echo(&msg);
            let response = echo_request.send().promise.await?;
            println!("{:?}", response.get());
            Ok(())
        })
        .await
}
