pub mod queries;

use mongodb::{options::ClientOptions, Client};

use crate::env;

pub use queries::*;
pub type MongoClient = Client;
pub type MongoError = mongodb::error::Error;

// todo: validate collections on init
pub async fn init() -> Result<MongoClient, MongoError> {
  let url = dotenvy::var(env::DB_URL).unwrap();
  info!("Connecting to {}", url);

  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq".into());
  opts.default_database = Some("bafoksqiyr3wxpf".into());

  let client = MongoClient::with_options(opts)?;

  Ok(client)
}
