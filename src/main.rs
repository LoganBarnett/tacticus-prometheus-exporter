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
use prometheus_client::encoding::{
  EncodeLabelSet,
  EncodeLabelValue,
  EncodeMetric,
};
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tacticus_poll::poll;
use tap::Tap;
use std::sync::Mutex;
use ::log::*;

#[derive(Debug)]
pub struct Metrics {
  guild_member_activity: Family<Vec<String>, Gauge>,
}

pub struct AppState {
  pub metrics: Metrics,
  pub registry: Registry,
}

pub fn register(registry: &mut Registry, metrics: &Metrics) -> () {
  let name = "name";
  let help = "help";
  // TODO: Figure out what this wants.  It's not obvious from looking at the
  // basic example found here:
  // https://docs.rs/prometheus-client/latest/prometheus_client/#examples
  // registry.register(name, help, metrics.guild_member_activity);
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
    metrics: Metrics {
      guild_member_activity: Family::<Vec<String>, Gauge>::default(),
    },
    registry: Registry::default(),
  };
  register(&mut app_state.registry, &app_state.metrics);
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
