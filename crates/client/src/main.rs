use std::sync::Arc;

use quinn::crypto::rustls::QuicClientConfig;
use rustls_pki_types::pem::PemObject;

mod actors;
mod cert;
mod env;
mod error;
mod prelude;

#[tokio::main]
async fn main() -> prelude::Result<()> {
  dotenv::dotenv().ok();
  pretty_env_logger::init();
  tokio_rustls::rustls::crypto::ring::default_provider()
    .install_default()
    .expect("Failed to install ring as the default crypto provider");

  let root_store = {
    let mut root_store = rustls::RootCertStore::empty();

    root_store
      .add(
        rustls_pki_types::CertificateDer::from_pem_file("../server/cert_chain.pem").expect("FUCK"),
      )
      .expect("FUCK");

    root_store
  };

  let crypto = {
    let mut crypto = rustls::ClientConfig::builder()
      .with_root_certificates(root_store)
      .with_no_client_auth();

    crypto.alpn_protocols = vec![b"hq-29".into()];

    crypto
  };

  let client_config = quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

  let endpoint = {
    let mut endpoint = quinn::Endpoint::client(env::get().client_addr)?;

    endpoint.set_default_client_config(client_config);

    endpoint
  };

  let dispatch = actors::dispatch::Handle::new();

  let connection = endpoint
    .connect(env::get().server_addr, &env::get().server_name)?
    .await?;
  let (send, recv) = connection.open_bi().await?;

  let inbound = actors::inbound::Handle::new(recv, dispatch);
  let outbound = actors::outbound::Handle::new(send);

  outbound.send(proto::client::Message::Meow).await;

  inbound.join().await;
  endpoint.wait_idle().await;

  Ok(())
}
