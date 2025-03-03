use crate::prelude::*;

struct Actor {
  endpoint: quinn::Endpoint,
  dispatch: super::dispatch::Handle,
}

impl Actor {
  async fn run(mut self) {
    while let Some(incoming) = self.endpoint.accept().await {
      log::info!("Accepting connection from {}", incoming.remote_address());

      let dispatch = self.dispatch.clone();
      tokio::spawn(async move {
        let (send, recv) = incoming
          .await
          .expect("Failed to accept incoming connection")
          .accept_bi()
          .await
          .expect("Failed to accept a bidirectional stream");

        let _outbound = super::outbound::Handle::new(send);
        let inbound = super::inbound::Handle::new(recv, dispatch.clone());

        inbound.join().await;
      });
    }
  }
}

pub struct Handle {
  join_handle: tokio::task::JoinHandle<()>,
}

impl Handle {
  pub fn new(endpoint: quinn::Endpoint, dispatch: super::dispatch::Handle) -> Self {
    let actor = Actor { endpoint, dispatch };

    let join_handle = tokio::spawn(async move { actor.run().await });

    Self { join_handle }
  }

  pub async fn join(self) {
    self.join_handle.await.expect("Listener actor panicked");
  }
}
