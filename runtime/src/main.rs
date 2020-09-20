use std::path::Path;

use entity::{EntityMsg, EntitySystem};
use log::debug;
use structopt::StructOpt;
use system::System;

mod cli;

struct Runtime;

impl System<()> for Runtime {
  fn system_addr(&self) -> system::Addr<()> {
    panic!()
  }

  fn startup(self) -> system::Addr<()> {
    panic!()
  }
}

pub fn main() {
  env_logger::init();
  debug!("starting runtime");
  let runtime = Runtime;

  debug!("getting CLI options");
  let cli = cli::CLI::from_args();

  debug!("instantiating entity system");
  let entity_system = EntitySystem::new(cli.entity_root_path);
  let entity_system_addr = entity_system.system_addr();
  entity_system.startup();

  std::thread::sleep_ms(10000);
  runtime.send_msg(entity_system_addr, EntityMsg::Kill);
  log::info!("quitting application, bye");
  std::thread::sleep_ms(1000);
}
