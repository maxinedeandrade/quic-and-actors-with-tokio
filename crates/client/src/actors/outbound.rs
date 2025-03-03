use tokio::{io::AsyncWriteExt, sync::mpsc};

const CHANNEL_SIZE: usize = 8;

struct Actor {
  stream: quinn::SendStream,
  rx: mpsc::Receiver<proto::client::Message>,
}

impl Actor {
  async fn run(mut self) {
    while let Some(msg) = self.rx.recv().await {
      self.send(msg).await;
    }
  }

  async fn send(&mut self, message: proto::client::Message) {
    let buffer = bitcode::encode(&message);

    self
      .stream
      .write_all(&buffer)
      .await
      .expect("Failed to write message to stream");

    self.stream.flush().await.expect("Failed to flush stream");
  }
}

pub struct Handle {
  tx: mpsc::Sender<proto::client::Message>,
}

impl Handle {
  pub fn new(stream: quinn::SendStream) -> Self {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

    tokio::spawn(Actor { stream, rx }.run());

    Self { tx }
  }

  pub async fn send(&self, message: proto::client::Message) {
    self.tx.send(message).await.expect("Failed to send message");
  }
}
