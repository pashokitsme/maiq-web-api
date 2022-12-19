#[macro_use]
extern crate rocket;

mod api;
mod handlers;

use api::{error::not_found, routes::*};

#[shuttle_service::main]
async fn run() -> shuttle_service::ShuttleRocket {
  let rocket = rocket::build()
    .register("/", catchers![not_found])
    .mount("/", routes![index])
    .mount("/api", routes![get_instantly]);

  Ok(rocket)
}
