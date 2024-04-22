fn main() {
    println!("running build script");
    capnpc::CompilerCommand::new()
        .src_prefix("proto")
        .file("proto/chain.capnp")
        .file("proto/common.capnp")
        .file("proto/echo.capnp")
        .file("proto/init.capnp")
        .file("proto/handler.capnp")
        .file("proto/node.capnp")
        .file("proto/proxy.capnp")
        .file("proto/wallet.capnp")
        .output_path("src/proto")
        .run()
        .expect("schema compiler command failed to run");

    println!("cargo:rerun-if-changed=proto/chain.capnp");
    println!("cargo:rerun-if-changed=proto/common.capnp");
    println!("cargo:rerun-if-changed=proto/echo.capnp");
    println!("cargo:rerun-if-changed=proto/init.capnp");
    println!("cargo:rerun-if-changed=proto/handler.capnp");
    println!("cargo:rerun-if-changed=proto/node.capnp");
    println!("cargo:rerun-if-changed=proto/proxy.capnp");
    println!("cargo:rerun-if-changed=proto/wallet.capnp");
}
