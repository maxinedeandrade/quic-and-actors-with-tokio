use tokio::sync::mpsc;

const CHANNEL_SIZE: usize = 8;

struct Actor {
  rx: mpsc::Receiver<proto::server::Message>,
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
  tx: mpsc::Sender<proto::server::Message>,
}

impl Handle {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
    let actor = Actor { rx };

    tokio::spawn(async move { actor.run().await });

    Self { tx }
  }

  pub async fn send(&self, message: proto::server::Message) {
    self
      .tx
      .send(message)
      .await
      .expect("Failed to send actor a message");
  }
}
