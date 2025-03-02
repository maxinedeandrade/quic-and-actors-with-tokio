use std::{
  net::{Ipv4Addr, SocketAddr},
  path::{Path, PathBuf},
  sync::OnceLock,
};

#[derive(Debug)]
pub struct Env {
  pub listen_addr: SocketAddr,
  pub cert_chain_path: PathBuf,
  pub key_path: PathBuf,
}

impl Env {
  const DEFAULT_LISTEN_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8456);
  const DEFAULT_CERT_CHAIN_PATH: &str = "cert_chain.pem";
  const DEFAULT_KEY_PATH: &str = "key.der";
}

pub fn get() -> &'static Env {
  static INSTANCE: OnceLock<Env> = OnceLock::new();

  INSTANCE.get_or_init(|| Env {
    listen_addr: std::env::var("SERVER_LISTEN_ADDR")
      .inspect_err(|e| {
        log::warn!(
          "Failed to read SERVER_LISTEN_ADDR environment variable: {:?}",
          e
        )
      })
      .ok()
      .and_then(|addr| {
        addr
          .parse()
          .inspect_err(|e| log::warn!("Failed to parse SERVER_LISTEN_ADDR: {:?}", e))
          .ok()
      })
      .unwrap_or(Env::DEFAULT_LISTEN_ADDR),
    cert_chain_path: std::env::var("SERVER_CERT_CHAIN_PATH")
      .map(PathBuf::from)
      .unwrap_or_else(|_| Env::DEFAULT_CERT_CHAIN_PATH.into()),
    key_path: std::env::var("SERVER_KEY_PATH")
      .map(PathBuf::from)
      .unwrap_or_else(|_| Env::DEFAULT_KEY_PATH.into()),
  })
}
