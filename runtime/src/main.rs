use colored::Colorize;
use entity::{EntityMsg, EntitySystem};
use log::debug;
use proto::RuntimeMsg;
use std::{collections::HashSet, path::Path};
use structopt::StructOpt;
use system::{system_init, Addr, MsgQueue, System, SystemUID};

mod cli;

/// Runtime system.
struct Runtime {
  systems: HashSet<SystemUID>,
  addr: Addr<RuntimeMsg>,
  messages: MsgQueue<RuntimeMsg>,
}

impl Runtime {
  fn new() -> Self {
    let (addr, msg_queue) = system_init();

    Runtime {
      systems: HashSet::new(),
      addr,
      messages: msg_queue,
    }
  }

  /// Create a new [`SystemUID`] that is being considered active as a system.
  fn create_system(&mut self, name: &str) -> SystemUID {
    let uid = SystemUID::new();

    log::info!(
      "creating new {} system {}",
      name.blue().bold(),
      uid.to_string().blue().bold()
    );
    self.systems.insert(uid);

    uid
  }
}

impl System<RuntimeMsg, ()> for Runtime {
  fn system_addr(&self) -> Addr<RuntimeMsg> {
    self.addr.clone()
  }

  fn startup(mut self) {
    env_logger::init();
    log::info!("starting runtime");

    log::debug!("getting CLI options");
    let cli = cli::CLI::from_args();

    let entity_uid = self.create_system("entity");
    let entity_system = EntitySystem::new(self.system_addr(), entity_uid, cli.entity_root_path);
    let entity_system_addr = entity_system.system_addr();
    entity_system.startup();

    // kill everything if we receive SIGINT
    ctrlc::set_handler(|| {}).unwrap();

    loop {
      match self.messages.recv() {
        Some(RuntimeMsg::SystemExit(uid)) => {
          log::info!("system {} has exited", uid.to_string().blue().bold());
          let _ = self.systems.remove(&uid);
        }
        None => {}
      }

      if self.systems.is_empty() {
        break;
      }
    }

    log::info!("all systems cleared; bye…");
  }

  fn publish(&self, _: ()) {}

  fn subscribe(&mut self, _: Addr<()>) {}
}

pub fn main() {
  Runtime::new().startup();
}
