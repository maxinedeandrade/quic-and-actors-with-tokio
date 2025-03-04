use super::dispatch;
use crate::actors::{inbound, outbound};
use tokio::task;

struct Actor {
  endpoint: quinn::Endpoint,
}

impl Actor {
  async fn run(mut self) {
    while let Some(incoming) = self.endpoint.accept().await {
      log::info!("Accepting connection from {}", incoming.remote_address());

      tokio::spawn(async move {
        let (send, recv) = incoming
          .await
          .expect("Failed to accept incoming connection")
          .accept_bi()
          .await
          .expect("Failed to accept a bidirectional stream");

        let outbound = outbound::Handle::new(send);
        let dispatch = dispatch::Handle::new(outbound);
        let inbound = inbound::Handle::new(recv, dispatch);

        inbound.join().await;
      });
    }
  }
}

pub struct Handle {
  join_handle: task::JoinHandle<()>,
}

impl Handle {
  pub fn new(endpoint: quinn::Endpoint) -> Self {
    let actor = Actor { endpoint };

    let join_handle = tokio::spawn(async move { actor.run().await });

    Self { join_handle }
  }

  pub async fn join(self) {
    self.join_handle.await.expect("Listener actor panicked");
  }
}
