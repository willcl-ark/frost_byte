use bitcoin_ipc::echo_capnp;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::proxy::ProxyClient;

#[derive(Clone)]
pub struct ThreadSafeEchoClient(Arc<Mutex<echo_capnp::echo::Client>>);

impl ThreadSafeEchoClient {
    pub fn new(client: echo_capnp::echo::Client) -> Self {
        ThreadSafeEchoClient(Arc::new(Mutex::new(client)))
    }

    pub async fn echo_request(&self) -> echo_capnp::echo::echo_request::Builder {
        let client = self.0.lock().await;
        client.echo_request()
    }
}

// Implement Send and Sync for ThreadSafeEchoClient
unsafe impl Send for ThreadSafeEchoClient {}
unsafe impl Sync for ThreadSafeEchoClient {}

pub struct EchoClientProxy {
    proxy: ProxyClient<ThreadSafeEchoClient>,
}

impl EchoClientProxy {
    pub async fn echo_request(
        &self,
        echo: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.proxy
            .invoke(|client| async move {
                let mut request = client.echo_request().await;
                request.get().set_echo(echo);
                request.send().promise.await
            })
            .await?;
        Ok(())
    }
}
