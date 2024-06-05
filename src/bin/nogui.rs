use frost_byte::spawner::LocalSpawner;
use frost_byte::tasks::Task;

#[tokio::main]
async fn main() {
    // Start cap'n'proto thread
    let spawner = LocalSpawner::new();

    // Send a task to the cap'n'proto thread
    let (send, response) = tokio::sync::oneshot::channel();
    spawner.spawn(Task::SetupConnection(send));
    match response.await {
        Ok(_) => panic!("Great success!"),
        Err(_) => println!("the sender dropped"),
    }
}
