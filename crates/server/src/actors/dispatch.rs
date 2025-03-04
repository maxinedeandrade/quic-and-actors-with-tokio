use super::outbound;
use tokio::sync::mpsc;

const CHANNEL_SIZE: usize = 16;

struct Actor {
  rx: mpsc::Receiver<proto::client::Message>,
  outbound: outbound::Handle,
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
  pub fn new(outbound: outbound::Handle) -> Self {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
    let actor = Actor { rx, outbound };

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
