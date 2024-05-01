use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use tokio::net::UnixStream;
use tokio_util::compat::*;

pub mod chain_capnp;
pub mod common_capnp;
pub mod handler_capnp;
pub mod proxy_capnp;
pub mod wallet_capnp;
use crate::chain_capnp::chain::Client;

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

            let mut request = frost_byte.get_height_request();
            request.get();
            let reply = request.send().promise.await?;

            println!(
                "received: {:?}",
                reply.get()? // reply.get()?.get_reply()?.get_message()?.to_str()?
            );
            Ok(())
        })
        .await
}
