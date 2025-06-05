use ::log::*;
use std::{thread::JoinHandle, time::Duration};

use reqwest::Client;

use crate::{apis::{configuration::{ApiKey, Configuration}, tacticus_player_api_api::get_player}, error::AppError};

pub fn poll(api_key: &String, frequency: u64) -> JoinHandle<()> {
  let api_key = api_key.clone();
  std::thread::spawn(move || {
    let client = Client::new();
    let config = Configuration {
      base_path: "https://api.tacticusgame.com".to_owned(),
      user_agent: Some("OpenAPI-Generator/0.1 BETA/rust".to_owned()),
      client,
      basic_auth: None,
      oauth_access_token: None,
      bearer_access_token: None,
      api_key: Some(ApiKey { prefix: None, key: api_key.to_string(), }),
    };
    loop {
      debug!("Polling API...");
      let player = tokio::runtime::Runtime::new().unwrap().block_on(
        get_player(&config)
      ).map_err(AppError::GetPlayerError).unwrap();
      let details = player.player.details;
      info!(
        "Got player data for {}.  Power level: {}",
        details.name,
        details.power_level,
      );
      std::thread::sleep(Duration::from_millis(frequency * 1000));
    }
  })
}
