#[derive(Debug)]
pub enum AppError {
  ApiKeyReadError(std::io::Error),
  CliParseError(clap::error::Error),
  HttpBindError(std::io::Error),
  LoggingInitializationError(log::SetLoggerError),
  GetPlayerError(crate::apis::Error<crate::apis::tacticus_player_api_api::GetPlayerError>),
}
