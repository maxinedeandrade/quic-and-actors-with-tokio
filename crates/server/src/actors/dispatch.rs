use crate::prelude::*;
use tokio::sync::{mpsc, oneshot};

const CHANNEL_SIZE: usize = 16;

struct Actor {
  rx: mpsc::Receiver<proto::client::Message>,
}

impl Actor {
  async fn run(mut self) {
    while let Some(message) = self.rx.recv().await {
      log::info!("Received message: {:?}", message);
    }
  }
}

#[derive(Clone)]
pub struct Handle {
  tx: mpsc::Sender<proto::client::Message>,
}

impl Handle {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
    let actor = Actor { rx };

    tokio::spawn(async move { actor.run().await });

    Self { tx }
  }

  pub async fn send(&self, message: proto::client::Message) {
    self
      .tx
      .send(message)
      .await
      .expect("Failed to send actor a message");
  }
}
