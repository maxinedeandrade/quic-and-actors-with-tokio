use tokio::{sync::mpsc, task};

const CHANNEL_SIZE: usize = 8;
const BUFFER_SIZE: usize = 1024 * 8;

struct Actor {
  stream: quinn::RecvStream,
  dispatch: super::dispatch::Handle,
}

impl Actor {
  async fn run(mut self) {
    let mut buffer = Box::new([0u8; BUFFER_SIZE]);

    loop {
      match self
        .stream
        .read(buffer.as_mut())
        .await
        .expect("Failed to read stream")
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
  join_handle: task::JoinHandle<()>,
}

impl Handle {
  pub fn new(stream: quinn::RecvStream, dispatch: super::dispatch::Handle) -> Self {
    let actor = Actor { stream, dispatch };

    let join_handle = tokio::spawn(async move { actor.run().await });

    Self { join_handle }
  }

  pub async fn join(self) {
    self.join_handle.await.expect("Failed to join actor");
  }
}
