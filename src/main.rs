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
            // Wait for the result.  This is the only line that blocks.
            let construct_response = construct_request.send().promise.await?;
            println!(
                "received construct reponse: {:?}",
                construct_response.get()?
            );

            // ----------------------------------------------------------------
            // Get a thread_map which also returns a proxy_capnp::thread_map::Client
            // Use this in future init_capnp::* calls perhaps?
            // ----------------------------------------------------------------
            let thread_map = construct_response.get()?.get_thread_map()?;
            let thread_request = thread_map.make_thread_request();
            let thread_response = thread_request.send().promise.await?;
            println!("received thread response: {:?}", thread_response.get()?);

            // ----------------------------------------------------------------
            // Create a MakeEcho object on the server
            // ----------------------------------------------------------------
            // Make an echo client request
            let mut make_echo_request = init_client.make_echo_request();
            // Set the context
            make_echo_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);

            // Wait for the response
            let echo_client_response = make_echo_request.send().promise.await?;
            println!(
                "received echo_client response: {:?}",
                echo_client_response.get()?
            );

            // ----------------------------------------------------------------
            // Make a new echo CLIENT
            // ----------------------------------------------------------------
            // We can reuse this client
            let echo_client = echo_client_response.get()?.get_result()?;

            let mut new_echo_request = echo_client.echo_request();
            new_echo_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            new_echo_request.get().set_echo("Hello, world!");
            let new_echo = new_echo_request.send().promise.await?;
            println!("received echo response: {:?}", new_echo.get()?);

            // ----------------------------------------------------------------
            // We want to make a new chain here... we also need to make globalArgs
            // The wallet needs:
            //     makeWalletLoader @4 (context :Proxy.Context, globalArgs :Common.GlobalArgs, chain :Chain.Chain) -> (result :Wallet.WalletLoader);
            // ----------------------------------------------------------------

            // Make a chain client request
            let mut make_chain_request = init_client.make_chain_request();
            // Set the context
            make_chain_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);

            // Wait for the response
            let chain_client_response = make_chain_request.send().promise.await?;
            println!(
                "received chain_client response: {:?}",
                chain_client_response.get()?
            );

            // We can reuse this client
            let chain_client = chain_client_response.get()?.get_result()?;
            let mut new_chain_request = chain_client.get_height_request();
            new_chain_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            let new_chain = new_chain_request.send().promise.await?;
            println!("received chain response: {:?}", new_chain.get()?);

            // Make a node client request
            //
            // We need a node here because the node is what keeps all of the global state, e.g.
            // our bitcoin.conf and any startup options passed
            let mut make_node_request = init_client.make_node_request();
            // Set the context
            make_node_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);

            // Wait for the response
            let node_client_response = make_node_request.send().promise.await?;
            println!(
                "received make_node_request response: {:?}",
                node_client_response.get()?
            );
            let node_client = node_client_response.get()?.get_result()?;

            // Now that I have a node, I can use custom_wallet_loader to have the node return
            // *its* wallet loader to me. This means the node process can use its internal state
            // (globalArgs) to create the wallet loader and then return the wallet loader to me
            // which then allows me to "remote control" the nodes wallet.
            //
            // If I wanted to make my own wallet, I would need to get the state from bitcoin.conf
            // or we need to update either the Init or Node capnp interfaces with a new method just
            // for returning global state (gargs, in this case).
            let mut custom_wallet_loader_request = node_client.custom_wallet_loader_request();
            custom_wallet_loader_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            let cwl_response = custom_wallet_loader_request.send().promise.await?;
            println!("received cwl request: {:?}", cwl_response.get()?);

            let mut create_wallet_request =
                cwl_response.get()?.get_result()?.create_wallet_request();
            create_wallet_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            create_wallet_request.get().set_name("frost_byte");
            let _create_wallet_response = create_wallet_request.send().promise.await?;

            // Check that our wallet was created
            let mut list_wallet_dir_request =
                cwl_response.get()?.get_result()?.list_wallet_dir_request();
            list_wallet_dir_request
                .get()
                .get_context()?
                .set_thread(thread_response.get()?.get_result()?);
            let list_wallet_dir_response = list_wallet_dir_request.send().promise.await?;
            println!(
                "got a list wallet dir response: {:?}",
                list_wallet_dir_response.get()?
            );

            Ok(())
        })
        .await
}
