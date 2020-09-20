use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CLI {
  #[structopt(short = "p", long, default_value = ".")]
  pub entity_root_path: PathBuf,
}
