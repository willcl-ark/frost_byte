use capnp::capability::{FromClientHook, FromServer};
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
            let binding = args[1].to_string();
            let path = std::path::Path::new(&binding);

            // Create bi-directional stream
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
            let init_client: init_capnp::init::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
            tokio::task::spawn_local(rpc_system);

            // Call construct first to get back a ThreadMap
            //
            //     pub fn construct_request(&self) -> ::capnp::capability::Request<crate::init_capnp::init::construct_params::Owned,crate::init_capnp::init::construct_results::Owned> {
            //         self.client.new_call(_private::TYPE_ID, 0, ::core::option::Option::None)
            //     }
            let mut construct_request = init_client.construct_request();
            construct_request.get();
            let construct_response = construct_request.send().promise.await?;
            println!("received: {:?}", construct_response.get()?);

            // Next we should do something with this threadmap ???
            // let threadmap_client = construct_response.get()?.get_thread_map()?;
            // let mut thread = threadmap_client.make_thread_request();
            // let response = thread.send().promise.await?;
            // println!("received: {:?}", response.get()?);
            // thread.get().get_cap().set_as_capability(frost_byte);

            // Make echo request, passing in the threadmap, or having "applied" it
            // let init_client_hook = init_client.as_client_hook();
            // let echo_client = echo_capnp::echo::Client::from_server(rpc_system);

            // let mut request = echo_client.echo_request();
            // request.get();
            // let reply = request.send().promise.await?;
            // println!("received: {:?}", reply.get()?);

            Ok(())
        })
        .await
}
