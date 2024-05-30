pub mod chain_capnp {
    include!("chain_capnp.rs");
}

pub mod common_capnp {
    include!("common_capnp.rs");
}

pub mod echo_capnp {
    include!("echo_capnp.rs");
}

pub mod init_capnp {
    include!("init_capnp.rs");
}

pub mod handler_capnp {
    include!("handler_capnp.rs");
}

pub mod node_capnp {
    include!("node_capnp.rs");
}

pub mod proxy_capnp {
    include!("proxy_capnp.rs");
}

pub mod wallet_capnp {
    include!("wallet_capnp.rs");
}

pub mod chain;
pub mod echo;
pub mod init;
pub mod node;
pub mod wallet;
