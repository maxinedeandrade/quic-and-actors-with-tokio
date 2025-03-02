use std::{net::SocketAddr, sync::OnceLock};

#[derive(Debug)]
pub struct Env {
  pub server_addr: SocketAddr,
  pub server_name: String,
}

pub fn get() -> &'static Env {
  static INSTANCE: OnceLock<Env> = OnceLock::new();

  INSTANCE.get_or_init(|| Env {
    server_addr: std::env::var("SERVER_LISTEN_ADDR")
      .expect("SERVER_LISTEN_ADDR must be set")
      .parse()
      .expect("SERVER_LISTEN_ADDR must be a valid SocketAddr"),
    server_name: std::env::var("SERVER_NAME").expect("SERVER_NAME must be set"),
  })
}
