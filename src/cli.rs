use clap::Parser;
use clap_verbosity_flag::Verbosity;

#[derive(Debug, Parser)]
#[command(about = "A Prometheus / OpenMetrics exporter for the Tacticus API.")]
pub struct Cli {
  // TODO: Default to something measured, like 10 minutes.
  #[arg(short, long, env)]
  pub poll_rate: u64,
  #[command(flatten)]
  pub verbosity: Verbosity,
  #[arg(short, long, env)]
  pub api_key_file: String,
}
