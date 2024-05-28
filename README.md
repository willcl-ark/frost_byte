# Setup

## Bitcoin Core

The cap'n'proto multi-process PR [#10102](https://github.com/bitcoin/bitcoin/pull/10102) of Bitcoin Core is required.
In addition, we currently use the `ipcbind` option introduced by [#19460](https://github.com/bitcoin/bitcoin/pull/19460).

You can either cherry-pick both together, or use [this branch](https://github.com/willcl-ark/bitcoin/tree/pr/ipc) which includes the latter cherry-picked onto the former.

Bitcoin Core requires the installation of [libmultiprocess](https://github.com/chaincodelabs/libmultiprocess).

After this, don't forget to configure Bitcoin Core with `./configure --enable-multiprocess` to build the `bitcoin-node` binary.

## Frost Byte

### Build proto files

To compile the cap'n'proto schema files into their rust bindings (needed to run Frost Byte), clone this project and run:

```bash
cargo build
```

All schema files found in `schema/` will have equivalent rust modules generated in the `src/` directory and be available to the rest of the project.

### Wallet creation test

This will create a new wallet called "frost_byte".
To run the test, you should first start `bitcoin-node` with the `-ipcbind=` option set, e.g.:

```bash
./src/bitcoin-node -regtest -ipc-connect=/home/user/.bitcoin/sockets/node.sock
```

Once `bitcoin-node` is running, you can run the Frost Byte test:

```bash
cargo run /home/user/.bitcoin/sockets/node.sock
```

You should see output in the terminal that various clients were successfully created and queried:

```log
₿ cargo run /home/user/.bitcoin/sockets/node.sock
   Compiling frost_byte v0.1.0 (/home/user/src/frost_byte)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.95s
     Running `target/debug/frost_byte /home/user/.bitcoin/sockets/node.sock`
received construct response: (threadMap = <external capability>)
received thread response: (result = <external capability>)
received echo_client response: (result = <external capability>)
received echo response: (result = "Hello, world!")
received chain_client response: (result = <external capability>)
received chain response: (result = 20054, hasResult = true)
received make_node_request response: (result = <external capability>)
received cwl request: (result = <external capability>)
got a list wallet dir response: (result = ["test", "frost_byte", "test-legacy", "default", "descriptor", "encrypted_blank", "big", "tmp_to_export"])
```
