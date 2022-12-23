#[macro_use]
extern crate rocket;

#[macro_use]
extern crate tracing;

mod api;
mod cache;
mod db;
mod env;

use api::{
  error::{internal_server_error, not_found},
  routes::*,
};
use cache::Cache;
use maiq_parser::Fetch;
use rocket::Error;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

#[rocket::main]
async fn main() -> Result<(), Error> {
  // db::init(&pool).await.map_err(CustomError::new)?;
  tracing_subscriber::fmt::init();
  env::check_env_vars();
  let pool = db::init().await.expect("Error while connecting to database");
  let cache = Arc::new(Mutex::new(Cache::default()));

  let pool_ref = pool.clone();
  let mut cache_ref = cache.clone();

  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_secs(60 * 5));
    let interval_chrono_duration = chrono::Duration::milliseconds(60 * 5 * 1000);
    loop {
      interval.tick().await;
      _ = cache::update(&pool_ref, Fetch::Today, &mut cache_ref, &interval_chrono_duration).await;
      _ = cache::update(&pool_ref, Fetch::Tomorrow, &mut cache_ref, &interval_chrono_duration).await;
    }
  });

  let pool_ref = pool.clone();
  let cache_ref = cache.clone();

  _ = rocket::build()
    .register("/", catchers![not_found, internal_server_error])
    .mount("/", routes![index])
    .mount("/api", routes![index, latest, poll, snapshot_by_id, naive])
    .manage(pool_ref)
    .manage(cache_ref)
    .launch()
    .await?;
  Ok(())
}
