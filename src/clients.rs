use bitcoin_ipc::echo_capnp::echo;
use bitcoin_ipc::init_capnp::init;
use bitcoin_ipc::node_capnp::node;
use bitcoin_ipc::proxy_capnp::thread;
use bitcoin_ipc::wallet_capnp::wallet_loader;
use std::sync::RwLock;

pub struct Clients {
    pub init_client: RwLock<Option<init::Client>>,
    pub thread_client: RwLock<Option<thread::Client>>,
    pub echo_client: RwLock<Option<echo::Client>>,
    pub node_client: RwLock<Option<node::Client>>,
    pub wallet_loader_client: RwLock<Option<wallet_loader::Client>>,
}

impl Clients {
    pub(crate) fn new() -> Self {
        Self {
            init_client: RwLock::new(None),
            thread_client: RwLock::new(None),
            echo_client: RwLock::new(None),
            node_client: RwLock::new(None),
            wallet_loader_client: RwLock::new(None),
        }
    }
}
