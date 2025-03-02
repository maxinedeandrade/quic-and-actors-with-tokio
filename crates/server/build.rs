fn main() -> Result<(), Box<dyn std::error::Error>> {
  if std::fs::exists("cert.pem")? && std::fs::exists("key.pem")? {
    return Ok(());
  }

  let rcgen::CertifiedKey { cert, key_pair } =
    rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])?;

  std::fs::write("cert.pem", &cert.pem())?;
  std::fs::write("key.pem", &key_pair.serialize_pem())?;

  Ok(())
}
