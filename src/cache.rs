use maiq_parser::{fetch_n_parse, Fetch};

use crate::{
  api::error::ApiError,
  db::{self, MongoClient},
};

pub async fn update(mongo: &MongoClient, fetch: Fetch) -> Result<(), ApiError> {
  info!("Updating cache for {:?}..", fetch);
  let latest = match fetch {
    Fetch::Today => db::get_latest_today(&mongo).await?,
    Fetch::Tomorrow => db::get_latest_next(&mongo).await?,
  };
  let snapshot = fetch_n_parse(fetch).await?.snapshot;
  if let Some(latest) = latest {
    info!("Comparing: {} & {}", snapshot.uid, latest.uid);
    if latest.uid == snapshot.uid {
      return Ok(());
    }
  }
  info!("Found new unique snapshot: #{}", snapshot.uid);
  db::save(&mongo, &snapshot).await?;
  Ok(())
}
