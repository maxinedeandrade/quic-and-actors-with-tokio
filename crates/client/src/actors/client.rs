use tokio::{io::AsyncReadExt, sync::mpsc};

use crate::{env, prelude::*};

const CHANNEL_SIZE: usize = 8;
const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;

struct SendActor {
  rx: mpsc::Receiver<proto::client::Message>,
  stream: quinn::SendStream,
}

impl SendActor {
  async fn run(mut self) {
    while let Some(msg) = self.rx.recv().await {}
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
  tx: mpsc::Sender<proto::client::Message>,
  join_handle: tokio::task::JoinHandle<()>,
}

impl Handle {
  pub async fn new(endpoint: quinn::Endpoint, dispatch: super::dispatch::Handle) -> Result<Self> {
    let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

    let (send, recv) = endpoint
      .connect(env::get().server_addr, &env::get().server_name)?
      .await?
      .open_bi()
      .await?;

    let send = SendActor { rx, stream: send };
    let recv = RecvActor {
      stream: recv,
      dispatch,
    };

    tokio::spawn(async move { send.run().await });
    let join_handle = tokio::spawn(async move { recv.run().await });

    Ok(Self { tx, join_handle })
  }

  pub async fn send(&self, message: proto::client::Message) {
    self
      .tx
      .send(message)
      .await
      .expect("Failed to send actor a message");
  }

  pub async fn join(self) {
    self.join_handle.await.expect("Client actor failed");
  }
}
