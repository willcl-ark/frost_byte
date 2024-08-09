# Setup

## Bitcoin Core

The cap'n'proto multi-process PR [#10102](https://github.com/bitcoin/bitcoin/pull/10102) of Bitcoin Core is required.
In addition, we currently use the `ipcbind` option introduced by [#19460](https://github.com/bitcoin/bitcoin/pull/19460).

You can either cherry-pick both together, or use [this branch](https://github.com/willcl-ark/bitcoin/tree/pr/ipc) which includes the latter cherry-picked onto the former.

Additionally, Bitcoin Core will depend on libmultiprocess and libcapnp.
These are both available in [depends](https://github.com/bitcoin/bitcoin/blob/master/depends/README.md).

To build Bitcoin Core with these libs you can something like:

```bash
cd /path/to/bitcoin/src
cd depends
make -j`nproc` NO_QT=1 NO_BDB=1 NO_QR=1 MULTIPROCESS=1
# Actual path to config.site differs per host-platform-triplet
CONFIG_SITE=$PWD/depends/x86_64-pc-linux-gnu/share/config.site ./configure
make -j`nproc`
```

Check the configure output along the way for multiprocess enabled:

```log
Options used to compile and link:
  external signer = yes
  multiprocess    = yes
```

## Frost Byte

### Build proto files

The cap'n'proto files from /schema will be automatically built as part of the cargo build by the bitcoin-ipc crate.
All schema files found in `schema/` will have equivalent rust modules generated in the `src/` directory and be available to the rest of the project.

### Demo run application

To run frost_byte you either need to start `bitcoin-node` manually and provide the socket address to it (else a default of `$HOME/.bitcoin/sockets/node.sock` is tried), or use the `--spawn` option:

```bash
# socket method
## start bitcoin-node with ipcbind
/path/to/src/bitcoin/src/bitcoin-node -ipcbind=unix:/$HOME/.bitcoin/sockets/node.sock -debug=ipc -daemon=0
## connect frost_byte
cargo run -- --socket $HOME/.bitcoin/sockets/node.sock

# auto-spawn method
cargo run -- --spawn /path/to/bitcoin/src/bitcoin-node
```

This will open a GUI window with two buttons to create different client connections.

