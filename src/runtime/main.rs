mod cli;

use colored::Colorize;
use luminance_windowing::WindowOpt;
use spectra::{
  entity::EntitySystem,
  graphics::GraphicsSystem,
  proto::Kill,
  runtime::RuntimeMsg,
  system::{system_init, Addr, MsgQueue, Publisher as _, System, SystemUID},
};
use std::{collections::HashSet, sync::mpsc::sync_channel, thread};
use structopt::StructOpt;

/// Runtime system.
struct Runtime {
  systems: HashSet<SystemUID>,
  addr: Addr<RuntimeMsg>,
  messages: MsgQueue<RuntimeMsg>,
}

impl Runtime {
  fn new() -> Self {
    let (addr, msg_queue) = system_init(SystemUID::new());

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
      name.blue(),
      uid.to_string().cyan().bold()
    );
    self.systems.insert(uid);

    uid
  }
}

impl System for Runtime {
  type Addr = Addr<RuntimeMsg>;

  fn system_addr(&self) -> Addr<RuntimeMsg> {
    self.addr.clone()
  }

  fn startup(mut self) {
    env_logger::init();

    log::debug!("getting CLI options");
    let cli = cli::CLI::from_args();

    // runtime system
    let runtime_uid = self.create_system("runtime");
    let runtime_system_addr = self.system_addr();

    // entity system
    let entity_uid = self.create_system("entity");
    let mut entity_system = EntitySystem::new(self.system_addr(), entity_uid, cli.entity_root_path);
    let entity_system_addr = entity_system.system_addr();

    // graphics system
    let graphics_uid = self.create_system("graphics");
    let graphics_system =
      GraphicsSystem::new(self.system_addr(), graphics_uid, WindowOpt::default()).unwrap();
    let graphics_system_addr = graphics_system.system_addr();
    entity_system.subscribe(graphics_system_addr.clone());

    // kill everything if we receive SIGINT
    let runtime_system_addr_ctrlc = runtime_system_addr.clone();
    ctrlc::set_handler(move || {
      runtime_system_addr_ctrlc.send_msg(Kill).unwrap();
    })
    .unwrap();

    entity_system.startup();

    // oneshot message to state that all systems have quit
    let (quit, has_quit) = sync_channel(1);

    // spawn the current entity in a different thread; this is needed because of the fact the graphics system
    // needs to run on the main thread (yeah I know it sucks)
    thread::spawn(move || loop {
      match self.messages.recv() {
        Some(RuntimeMsg::Kill) => {
          let _ = entity_system_addr.send_msg(Kill);
          let _ = graphics_system_addr.send_msg(Kill);
          // let _ = self.systems.remove(&runtime_uid);
          runtime_system_addr.send_msg(RuntimeMsg::SystemExit(runtime_uid));
        }

        Some(RuntimeMsg::SystemExit(uid)) => {
          log::info!("system {} has exited", uid.to_string().cyan().bold());
          let _ = self.systems.remove(&uid);
        }

        None => {}
      }

      if self.systems.is_empty() {
        log::info!("all systems cleared; bye…");
        quit.send(()).unwrap();
        break;
      }
    });

    graphics_system.startup();

    // before completely quitting, we need to be sure everybody quit
    has_quit.recv().unwrap();
  }
}

pub fn main() {
  Runtime::new().startup();
}
