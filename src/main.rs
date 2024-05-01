use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use tokio::net::UnixStream;
use tokio_util::compat::*;

pub mod chain_capnp;
pub mod common_capnp;
pub mod echo_capnp;
pub mod handler_capnp;
pub mod init_capnp;
pub mod node_capnp;
pub mod proxy_capnp;
pub mod wallet_capnp;
use crate::init_capnp::init::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            let args: Vec<String> = ::std::env::args().collect();
            if args.len() != 2 {
                println!("Usage:\n\t{} [path]", args[0]);
                return Ok(());
            }
            let binding = args[1].to_string();
            let path = std::path::Path::new(&binding);
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

            // I think we should perhaps make a context here and later apply the thread to it?
            // let context = proxy_capnp::context::Builder::get_thread

            // Call construct first to get back a ThreadMap
            let mut request = frost_byte.construct_request();
            request.get();
            let reply = request.send().promise.await?;
            println!("received: {:?}", reply.get()?);

            // Call echo
            let mut request = frost_byte.make_echo_request();
            request.get();
            let reply = request.send().promise.await?;
            println!("received: {:?}", reply.get()?);

            Ok(())
        })
        .await
}
