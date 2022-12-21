use maiq_parser::{fetch_n_parse, Fetch};

use crate::{
  api::error::ApiError,
  db::{self, MongoClient},
};

pub async fn update(mongo: &MongoClient, fetch: Fetch) -> Result<(), ApiError> {
  info!("Updating cache for {:?}..", fetch);

  let snapshot = fetch_n_parse(&fetch).await?.snapshot;
  let latest = db::get_by_uid(&mongo, snapshot.uid.as_str()).await?;
  if let Some(latest) = latest {
    return Ok(());
  }
  info!("New snapshot: #{}", snapshot.uid);
  db::save(&mongo, &snapshot).await?;
  Ok(())
}
