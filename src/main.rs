mod cli;
mod error;
mod tacticus_poll;
mod apis;
mod models;
mod logger;

use actix_web::middleware::Compress;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use clap::Parser;
use cli::Cli;
use error::AppError;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use tacticus_poll::poll;
use tap::Tap;
use std::sync::Mutex;
use ::log::*;

pub struct Metrics {

}

pub struct AppState {
  pub metrics: Metrics,
  pub registry: Registry,
}

pub async fn metrics_handler(
  state: web::Data<Mutex<AppState>>,
) -> Result<HttpResponse> {
  let state = state.lock().unwrap();
  let mut body = String::new();
  encode(&mut body, &state.registry).unwrap();
  Ok(
    HttpResponse::Ok()
      .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
      .body(body)
  )
}

#[actix_web::main]
async fn main() -> Result<(), AppError> {
  let cli_args = Cli::try_parse()
    .map_err(AppError::CliParseError)?;
  let api_key = std::fs::read_to_string(cli_args.api_key_file)
    .map_err(AppError::ApiKeyReadError)
    ?;
  logger::logger_init(&cli_args.verbosity)?;
  poll(&api_key.into(), cli_args.poll_rate);
  let mut app_state = AppState {
    metrics: Metrics {},
    registry: Registry::default(),
  };
  let app_data = web::Data::new(Mutex::new(app_state));
  let bind_address = "0.0.0.0";
  let port = 8080;
  HttpServer::new(move || {
    App::new()
      .app_data(app_data.clone())
      .service(web::resource("/metrics").route(web::get().to(metrics_handler)))
  })
    .bind((bind_address, port))
    .map_err(AppError::HttpBindError)?
    .tap(|_| info!("Listening on {}:{}", bind_address, port) )
    .run()
    .await
    ;
  Ok(())
}
