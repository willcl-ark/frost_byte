# Aims

## General

- GUI application can start node on it's own (bundled like Bitcoin-QT)
    - Something like `./src/bitcoin-node -ipcbind=unix://home/will/.bitcoin/sockets/node.sock -debug=ipc -regtest -daemon=0`
    - Can later demo connecting to a remote node

## Will todo

    - On launch:
    - Show that we're connected to the node with traffic light?
    - Corner widget showing sync progress/number peers etc., after creating `ChainClient`:
        - See https://github.com/ryanofsky/bitcoin/blob/45ef1e42f8c71c724e4b091f1024545364716761/src/ipc/capnp/node.capnp#L62C1-L68C120 for example

    ```
    Blocks: 850536
    Headers: 850536
    Verification progress: 99.9998%
    Difficulty: 83675262295059.91

    Network: in 10, out 12, total 22
    Version: 279900
    ```

- GUI can call `listWalletDir` and get a list of available wallets
    - show existing wallets via `listWallets` in a drop down
    - have a "Create new wallet" button via `WalletLoader::createWallet`
        - Create wallet pop-up with various options

- Now at "homepage" with "balance", "send", "receive"
- Make a "Get new address" button


## Josie

- Should move RPC code from `importdescriptors` into the src/wallet/interfaces.cpp file, so we can call from RPC and ipc
    - Maybe use `verifychain` instead, so we don't have to implement a new "Progress" callback, we can use:

    ```
    interface ShowWalletProgressCallback $Proxy.wrap("ProxyCallback<interfaces::Wallet::ShowProgressFn>") {
    destroy @0 (context :Proxy.Context) -> ();
    call @1 (context :Proxy.Context, title :Text, progress :Int32) -> ();
    }
    ```
    - Add a callback to `CWallet::ScanForWalletTransactions` which sent back progress every 100 blocks
        - Or equivalent for `verifychain`

- Subscribe to `updatedBlockTip` and pop-up on a new block arriving

## Reproducible builds

! @josie


## Misc

* Move IPC stuff into bitcoin-ipc crate
* Add miner-ipc ptoto files
* Document bitcoin-ipc

## Overview discussion questions

* How do kernel and MP fit together?
* Overall design doc
* Security implications/tradeoffs
