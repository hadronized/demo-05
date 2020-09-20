//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod mesh;

use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::thread;
use system::resource::ResourceManager;
use system::{system_init, Addr, MsgQueue, System};

use crate::mesh::Mesh;

/// All possible entities.
#[derive(Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Mesh),
}

#[derive(Debug)]
pub enum EntityMsg {}

/// The [`Entity`] system.
#[derive(Debug)]
pub struct EntitySystem {
  /// Directory where all scarce resources this entity system knows about live in.
  root_dir: PathBuf,
  resources: ResourceManager<Entity>,
  addr: Addr<EntityMsg>,
  msg_queue: MsgQueue<EntityMsg>,
}

impl EntitySystem {
  /// Create a new [`EntitySystem`].
  pub fn new(root_dir: impl AsRef<Path>) -> Self {
    let (addr, msg_queue) = system_init();

    Self {
      root_dir: root_dir.as_ref().to_owned(),
      resources: ResourceManager::new(),
      addr,
      msg_queue,
    }
  }

  /// Start the system.
  ///
  /// This method will first tries to load all the resources it can from `root_dir`, then will stay in an idle mode where it will:
  ///
  /// - Wait for system messages.
  /// - Wait for filesystem notifications.
  pub fn start(mut self) {
    let root_dir = self.root_dir.clone(); // TODO: check how we can remove the clone
    self.traverse_directory(&root_dir);
  }

  fn traverse_directory(&mut self, path: &Path) {
    log::debug!("traversing {}", path.display());

    for dir_entry in read_dir(path).unwrap() {
      let file = dir_entry.unwrap();
      let path = file.path();

      if path.is_dir() {
        // recursively traverse this repository
        self.traverse_directory(&path);
      } else if path.is_file() {
        // invoke a handler for this file
        log::info!("found resource file {}", path.display());
      }
    }
  }
}

impl System<EntityMsg> for EntitySystem {
  fn system_addr(&self) -> Addr<EntityMsg> {
    self.addr.clone()
  }

  fn startup(self) -> Addr<EntityMsg> {
    let addr = self.addr.clone();

    // move into a thread for greater good
    let _ = thread::spawn(move || {
      self.start();
    });

    addr
  }
}
