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
        let _client = super::client::Handle::new(incoming, dispatch);
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
    self.join_handle.await.expect("Listener actor failed");
  }
}
