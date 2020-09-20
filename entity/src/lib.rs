//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod mesh;

use colored::Colorize as _;
use std::ffi::OsStr;
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
pub enum EntityMsg {
  /// Kill message.
  Kill,
}

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

    // main loop
    loop {
      match self.msg_queue.recv() {
        Some(EntityMsg::Kill) | None => {
          log::info!("exiting system…");
          break;
        }

        _ => (),
      }
    }
  }

  fn traverse_directory(&mut self, path: &Path) {
    log::debug!(
      "traversing {}",
      path.display().to_string().purple().italic(),
    );

    for dir_entry in read_dir(path).unwrap() {
      let file = dir_entry.unwrap();
      let path = file.path();

      if path.is_dir() {
        // recursively traverse this repository
        self.traverse_directory(&path);
      } else if path.is_file() {
        // invoke a handler for this file
        log::info!(
          "found resource file {}",
          path.display().to_string().purple().italic(),
        );

        // extract the extension
        match path.extension().and_then(OsStr::to_str) {
          Some(ext) => {
            self.extension_based_dispatch(ext, &path);
          }

          None => {
            log::warn!(
              "resource {} doesn’t have a path extension; ignoring",
              path.display().to_string().purple().italic(),
            );
          }
        }
      }
    }
  }

  /// Dispatch entity loading based on the extension of a file.
  fn extension_based_dispatch(&mut self, ext: &str, path: &Path) {
    match ext {
      "obj" => self.load_obj(path),
      _ => log::warn!(
        "unknown extension {} for path {}",
        ext.blue().italic(),
        path.display().to_string().purple().italic(),
      ),
    }
  }

  /// Load .obj files.
  fn load_obj(&mut self, path: &Path) {
    match Mesh::load_from_path(path) {
      Ok(mesh) => {
        let path_name = path.display().to_string();
        let path = path_name.purple().italic();
        log::info!("{} {}", "loaded".green().bold(), path);

        let h = self.resources.wrap(Entity::Mesh(mesh), path_name);
        log::debug!("assigned {} handle {}", path, h.to_string().green().bold());
      }

      Err(err) => {
        log::error!(
          "cannot load OBJ {}: {}",
          path.display().to_string().purple().italic(),
          err,
        );
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
