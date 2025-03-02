use std::sync::Arc;

use quinn::crypto::rustls::QuicClientConfig;

mod actors;
mod cert;
mod env;
mod error;
mod prelude;

#[tokio::main]
async fn main() -> prelude::Result<()> {
  dotenv::dotenv().ok();
  pretty_env_logger::init();

  let crypto = {
    let mut crypto = rustls::ClientConfig::builder()
      .dangerous()
      .with_custom_certificate_verifier(Arc::new(cert::AssumeTrustworthy))
      .with_no_client_auth();

    crypto.alpn_protocols = vec![b"hq-29".into()];

    crypto
  };

  let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

  let endpoint = {
    let mut endpoint = quinn::Endpoint::client(env::get().server_addr)?;

    endpoint.set_default_client_config(client_config);

    endpoint
  };

  let dispatch = actors::dispatch::Handle::new();
  let client = actors::client::Handle::new(endpoint, dispatch).await?;

  client.join().await;

  Ok(())
}
