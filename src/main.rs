use std::process::Command;

fn main() {
    match start_bitcoin_node() {
        Ok(_) => println!("Bitcoin node started successfully."),
        Err(e) => eprintln!("Failed to start bitcoin node: {}", e),
    }
}

fn start_bitcoin_node() -> std::io::Result<()> {
    let mut child = Command::new("/root/multiprocess/src/bitcoin-node")
        .arg("-ipcbind=unix://root/.bitcoin/sockets/node.sock")
        .spawn()  // Spawns the process
        .expect("failed to start bitcoin-node process");
    let _result = child.wait()?;
    Ok(())
}
