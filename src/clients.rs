use bitcoin_ipc::echo_capnp::echo;
use bitcoin_ipc::init_capnp::init;
use bitcoin_ipc::proxy_capnp::thread;
use std::sync::RwLock;

pub struct Clients {
    pub init_client: RwLock<Option<init::Client>>,
    pub thread_client: RwLock<Option<thread::Client>>,
    pub echo_client: RwLock<Option<echo::Client>>,
}

impl Clients {
    pub(crate) fn new() -> Self {
        Self {
            init_client: RwLock::new(None),
            thread_client: RwLock::new(None),
            echo_client: RwLock::new(None),
        }
    }
}
