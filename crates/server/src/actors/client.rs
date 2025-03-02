use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};

use crate::prelude::*;

const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;
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
      .write_u32_le(buffer.len() as u32)
      .await
      .expect("Failed to write u32 to stream");

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
      let read = self
        .stream
        .read_u32_le()
        .await
        .expect("Failed to read u32 from stream") as usize;

      if read == 0 {
        continue;
      }

      self
        .stream
        .read_exact(buffer.as_mut())
        .await
        .expect("Failed to read {} bytes");

      log::info!("Received {} bytes", read);

      let client_message =
        bitcode::decode(&buffer[..read]).expect("Failed to decode client message");

      self.dispatch.send(client_message).await;
    }
  }
}

pub struct Handle {
  tx: mpsc::Sender<proto::server::Message>,
}

impl Handle {
  pub async fn new(incoming: quinn::Incoming, dispatch: super::dispatch::Handle) -> Result<Self> {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
    let (send, recv) = incoming.await?.open_bi().await?;

    let send = SendActor { stream: send, rx };
    let recv = RecvActor {
      stream: recv,
      dispatch,
    };

    tokio::spawn(async move { send.run().await });
    tokio::spawn(async move { recv.run().await });

    Ok(Self { tx })
  }
}
