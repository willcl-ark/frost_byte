use capnp::capability::FromClientHook;
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

            // Create bi-directional unix socket stream
            let stream = UnixStream::connect(path).await?;
            let (reader, writer) = stream.into_split();
            let reader_compat = reader.compat();
            let writer_compat = writer.compat_write();

            // Cap'n Proto RPC takes place between "vats".  A vat hosts some set of objects and talks to other
            // vats through direct bilateral connections.  Typically, there is a 1:1 correspondence between vats
            // and processes (in the unix sense of the word), although this is not strictly always true (one
            // process could run multiple vats, or a distributed virtual vat might live across many processes).
            //
            // The object lives on the "client" or "confined app" end of the connection.
            let rpc_network = Box::new(twoparty::VatNetwork::new(
                reader_compat,
                writer_compat,
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));

            //  Get the "bootstrap" interface exported by the remote vat.
            //
            //  For level 0, 1, and 2 implementations, the "bootstrap" interface is simply the main interface
            //  exported by a vat. If the vat acts as a server fielding connections from clients, then the
            //  bootstrap interface defines the basic functionality available to a client when it connects.
            //  The exact interface definition obviously depends on the application.
            let mut rpc_system = RpcSystem::new(rpc_network, None);

            // A network consisting of two vats.
            //
            // The generated Client type represents a reference to a remote Server. Clients are
            // pass-by-value types that use reference counting under the hood. (Warning: For
            // performance reasons, the reference counting used by Clients is not thread-safe, so
            // you must not copy a Client to another thread, unless you do it by means of an
            // inter-thread RPC.)
            //
            // For each interface method foo(), the Client has a method fooRequest() which creates
            // a new request to call foo(). The returned capnp::Request object has methods
            // equivalent to a Builder for the parameter struct (FooParams), with the addition of a
            // method send(). send() sends the RPC and returns a capnp::RemotePromise<FooResults>.

            // This RemotePromise is equivalent to kj::Promise<capnp::Response<FooResults>>, but
            // also has methods that allow pipelining. Namely:
            //
            //     For each interface-typed result, it has a getter method which returns a Client
            //     of that type. Calling this client will send a pipelined call to the server.
            //
            //     For each struct-typed result, it has a getter method which returns an object
            //     containing pipeline getters for that struct’s fields.
            //
            // In other words, the RemotePromise effectively implements a subset of the eventual
            // results’ Reader interface – one that only allows access to interfaces and
            // sub-structs.
            //
            // The RemotePromise eventually resolves to capnp::Response<FooResults>, which behaves
            // like a Reader for the result struct except that it also owns the result message.
            let init_client: init_capnp::init::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            // Spawn as a thread
            tokio::task::spawn_local(rpc_system);

            // Make a call to the capability.
            let construct_request = init_client.construct_request();

            // Optionally set some params
            // construct_request.set();

            // This gets a params builder I think?
            // construct_request.get();

            // Wait for the result.  This is the only line that blocks.
            let construct_response = construct_request.send().promise.await?;

            // Got a response.
            println!(
                "received construct reponse: {:?}",
                construct_response.get()?
            );

            // Get a thread_map which also returns a proxy_capnp::thread_map::Client
            // Use this in future init_capnp::* calls perhaps?
            let thread_map = construct_response.get()?.get_thread_map()?;
            let thread_request = thread_map.make_thread_request();
            let thread_response = thread_request.send().promise.await?;
            println!("received thread response: {:?}", thread_response.get()?);

            // Make a new echo request
            let mut echo_request = init_client.make_echo_request();
            // Get the request
            let params = echo_request.get();
            // Get the context and set the thread on it
            params
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            // Wait for the response
            let reply = echo_request.send().promise.await?;
            println!("received echo response: {:?}", reply.get()?);

            // Make a new wallet request
            // Ok so, the wallet needs:
            //
            //     makeWalletLoader @4 (context :Proxy.Context, globalArgs :Common.GlobalArgs, chain :Chain.Chain) -> (result :Wallet.WalletLoader);
            //
            // bitcoin-node is already started for us and has args and chain. So i presume we need to fetch them somehow. Going to leave for now.
            let mut wallet_request = init_client.make_wallet_loader_request();
            // Get the request
            let wallet_params = wallet_request.get();
            // Get the context and set the thread on it
            wallet_params
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            // Wait for the response
            let wallet_reply = wallet_request.send().promise.await?;
            println!("received wallet response: {:?}", wallet_reply.get()?);

            Ok(())
        })
        .await
}
