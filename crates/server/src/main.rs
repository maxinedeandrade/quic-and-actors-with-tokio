use quinn::crypto::rustls::QuicServerConfig;
use std::sync::Arc;

mod actors;
mod env;
mod error;
mod prelude;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
  dotenv::dotenv().ok();
  pretty_env_logger::init();
  tokio_rustls::rustls::crypto::ring::default_provider()
    .install_default()
    .expect("Failed to install ring as the default crypto provider");

  let cert_chain = {
    let cert_chain = std::fs::read(&env::get().cert_chain_path)?;
    rustls_pemfile::certs(&mut cert_chain.as_slice()).collect::<Result<_, _>>()?
  };

  let key_der = {
    let key_der = std::fs::read(&env::get().key_path)?;
    rustls_pemfile::private_key(&mut key_der.as_slice())?.expect("no private key found")
  };

  let crypto = {
    let mut crypto = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(cert_chain, key_der)?;

    crypto.alpn_protocols = vec![b"hq-29".into()];

    crypto
  };

  let server_config =
    quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));

  log::info!("Preparing to listen on {}", env::get().listen_addr);

  let endpoint = quinn::Endpoint::server(server_config, env::get().listen_addr)?;

  log::info!("Listening on {}", endpoint.local_addr()?);

  let dispatch = actors::dispatch::Handle::new();
  let listener = actors::listener::Handle::new(endpoint.clone(), dispatch);

  listener.join().await;
  endpoint.wait_idle().await;

  Ok(())
}
