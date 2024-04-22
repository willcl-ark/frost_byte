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
        .file("proto/wallet.capnp")
        .output_path("src/proto")
        .run()
        .expect("schema compiler command failed to run");

    println!("cargo:rerun-if-changed=schema/proto/chain.capnp");
    println!("cargo:rerun-if-changed=schema/proto/common.capnp");
    println!("cargo:rerun-if-changed=schema/proto/echo.capnp");
    println!("cargo:rerun-if-changed=schema/proto/init.capnp");
    println!("cargo:rerun-if-changed=schema/proto/handler.capnp");
    println!("cargo:rerun-if-changed=schema/proto/node.capnp");
    println!("cargo:rerun-if-changed=schema/proto/wallet.capnp");
}
