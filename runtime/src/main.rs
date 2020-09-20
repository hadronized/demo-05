use std::path::Path;

use entity::EntitySystem;
use log::debug;
use structopt::StructOpt;
use system::System;

mod cli;

pub fn main() {
  env_logger::init();
  debug!("starting runtime");

  debug!("getting CLI options");
  let cli = cli::CLI::from_args();

  debug!("instantiating entity system");
  let entity_system = EntitySystem::new(cli.entity_root_path);
  entity_system.startup();

  std::thread::sleep_ms(10000);
}
