#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("I/O error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("TLS error: {0}")]
  TlsError(#[from] rustls::Error),
  #[error("QUIC error: {0}")]
  NoInitialCipherSuite(#[from] quinn::crypto::rustls::NoInitialCipherSuite),
  #[error("QUIC connection error: {0}")]
  ConnectionError(#[from] quinn::ConnectionError),
}
