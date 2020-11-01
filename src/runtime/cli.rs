use std::path::PathBuf;
use structopt::StructOpt;

#[non_exhaustive]
#[derive(Debug, StructOpt)]
pub struct CLI {
  #[structopt(short = "r", long, default_value = ".")]
  pub entity_root_path: PathBuf,
}
