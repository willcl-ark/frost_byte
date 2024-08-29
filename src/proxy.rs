use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tokio::sync::RwLock;

struct Connection {
    handle: Handle,
}

impl Connection {
    fn new(handle: Handle) -> Self {
        Connection { handle }
    }

    async fn run_on_event_loop<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        self.handle.spawn(async move {
            let result = f().await;
            let _ = tx.send(result);
        });
        rx.await.expect("Task panicked")
    }
}

pub struct ProxyClient<T: Send + Sync + 'static> {
    client: Arc<RwLock<T>>,
    connection: Arc<Connection>,
}

impl<T: Send + Sync + 'static> ProxyClient<T> {
    fn new(client: T, connection: Arc<Connection>) -> Self {
        ProxyClient {
            client: Arc::new(RwLock::new(client)),
            connection,
        }
    }

    pub async fn invoke<F, Fut, R>(
        &self,
        f: F,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let client = self.client.clone();
        let connection = self.connection.clone();

        connection
            .run_on_event_loop(|| async move {
                let client = client.read().await;
                f(&client).await
            })
            .await
    }
}
