#[macro_use]
extern crate rocket;

#[macro_use]
extern crate tracing;

mod api;
mod cache;
mod db;
mod env;

use api::{error::not_found, routes::*};
use maiq_parser::Fetch;
use rocket::Error;
use std::time::Duration;
use tokio::time;

#[rocket::main]
async fn main() -> Result<(), Error> {
  // db::init(&pool).await.map_err(CustomError::new)?;
  tracing_subscriber::fmt::init();
  env::check_env_vars();
  let pool = db::init().await.expect("Error while connecting to database");
  let pool_state = pool.clone();

  tokio::spawn(async move {
    let mut interval = time::interval(Duration::from_secs(60 * 5));
    let mut interval_between = time::interval(Duration::from_secs(15));
    loop {
      interval.tick().await;
      _ = cache::update(&pool, Fetch::Today).await;
      interval_between.tick().await;
      _ = cache::update(&pool, Fetch::Tomorrow).await;
    }
  });

  _ = rocket::build()
    .register("/", catchers![not_found])
    .mount("/", routes![index])
    .mount("/api/poll/", routes![index, today, next])
    .mount("/api/dev/", routes![naive, update])
    .manage(pool_state)
    .launch()
    .await?;
  Ok(())
}
