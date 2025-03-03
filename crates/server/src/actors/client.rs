use tokio::{io::AsyncWriteExt, sync::mpsc};
use crate::prelude::*;

const DEFAULT_BUFFER_SIZE: usize = 1024 * 8;
const CHANNEL_SIZE: usize = 8;

struct SendActor {
  rx: mpsc::Receiver<proto::server::Message>,
  stream: quinn::SendStream,
}

impl SendActor {
  async fn run(mut self) {
    while let Some(msg) = self.rx.recv().await {
      self.send(msg).await;
    }
  }

  async fn send(&mut self, message: proto::server::Message) {
    let buffer = bitcode::encode(&message);

    self
      .stream
      .write_all(&buffer)
      .await
      .expect("Failed to write message to stream");

    self.stream.flush().await.expect("Failed to flush stream");
  }
}

struct RecvActor {
  stream: quinn::RecvStream,
  dispatch: super::dispatch::Handle,
}

impl RecvActor {
  async fn run(mut self) {
    let mut buffer = Box::new([0u8; DEFAULT_BUFFER_SIZE]);

    loop {
      match self
        .stream
        .read(buffer.as_mut())
        .await
        .expect("Failed to read from stream")
      {
        Some(0) | None => continue,
        Some(read) => {
          let client_message =
            bitcode::decode(&buffer[..read]).expect("Failed to decode ClientMessage");

          self.dispatch.send(client_message).await;
        }
      }
    }
  }
}

pub struct Handle {
  tx: mpsc::Sender<proto::server::Message>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl Handle {
  pub async fn new(incoming: quinn::Incoming, dispatch: super::dispatch::Handle) -> Result<Self> {
    let incoming = incoming.await?;
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
    let (send, recv) = incoming.accept_bi().await?;

    let send = SendActor { stream: send, rx };
    let recv = RecvActor {
      stream: recv,
      dispatch,
    };

    let join_handle = tokio::spawn(async move { recv.run().await });
    tokio::spawn(async move { send.run().await });

    Ok(Self { tx, join_handle })
  }

  pub async fn send(&self, message: proto::server::Message) {
    self
      .tx
      .send(message)
      .await
      .expect("Failed to send actor a message");
  }

  pub async fn join(self) {
    self.join_handle.await.expect("Client actor panicked");
  }
}
