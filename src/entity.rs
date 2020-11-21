//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod mesh;
pub mod parameter;

use crate::{
  proto::Kill,
  runtime::RuntimeMsg,
  system::resource::Handle,
  system::{
    resource::ResourceManager, system_init, Addr, MsgQueue, Publisher, Recipient, System, SystemUID,
  },
};
use colored::Colorize as _;
use mesh::Mesh;
use std::{
  ffi::OsStr,
  fs::read_dir,
  path::{Path, PathBuf},
  sync::Arc,
  thread,
};

/// All possible entities.
#[derive(Clone, Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Arc<Mesh>),
  /// A [`Parameter`].
  Parameter(self::parameter::Parameter),
}

#[derive(Clone, Debug)]
pub enum EntityMsg {
  /// Kill message.
  Kill,
}

impl From<Kill> for EntityMsg {
  fn from(_: Kill) -> Self {
    Self::Kill
  }
}

/// Event the entity system can emit.
#[derive(Clone, Debug)]
pub enum EntityEvent {
  /// A new entity was loaded / reloaded.
  Loaded {
    handle: Handle<Entity>,
    entity: Entity,
  },
}

/// The [`Entity`] system.
pub struct EntitySystem {
  uid: SystemUID,
  runtime_addr: Addr<RuntimeMsg>,
  /// Directory where all scarce resources this entity system knows about live in.
  root_dir: PathBuf,
  resources: ResourceManager<Entity>,
  addr: Addr<EntityMsg>,
  msg_queue: MsgQueue<EntityMsg>,
  subscribers: Vec<Box<dyn Recipient<EntityEvent>>>,
}

impl EntitySystem {
  /// Create a new [`EntitySystem`].
  pub fn new(runtime_addr: Addr<RuntimeMsg>, uid: SystemUID, root_dir: impl AsRef<Path>) -> Self {
    let (addr, msg_queue) = system_init(uid);

    Self {
      uid,
      runtime_addr,
      root_dir: root_dir.as_ref().to_owned(),
      resources: ResourceManager::new(),
      addr,
      msg_queue,
      subscribers: Vec::new(),
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
          self
            .runtime_addr
            .send_msg(RuntimeMsg::SystemExit(self.uid))
            .unwrap();
          break;
        }
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
        log::debug!(
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
      "json" => self.dispatch_json(path),
      _ => log::warn!(
        "unknown extension {} for path {}",
        ext.yellow().italic(),
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
        let mesh = Entity::Mesh(Arc::new(mesh));
        let handle = self.resources.wrap(mesh.clone(), path_name);

        log::debug!("assigned {} handle {}", path, handle);
        log::info!("{} mesh {} at {}", "loaded".green().bold(), handle, path);

        let event = EntityEvent::Loaded {
          handle,
          entity: mesh,
        };
        self.publish(event);
      }

      Err(err) => {
        log::error!(
          "cannot load {} {}: {}",
          "obj".yellow().italic(),
          path.display().to_string().purple().italic(),
          err,
        );
      }
    }
  }

  fn dispatch_json(&mut self, path: &Path) {
    // extract the “sub” extension, e.g. foo.bar.json’s sub extension is bar.
    match Self::extract_sub_extension(path) {
      Some(sub_ext) => match sub_ext {
        "param" => self.load_parameters(path),

        _ => {
          log::warn!(
            "unknown JSON {} resource for {}",
            sub_ext.yellow().italic(),
            path.display().to_string().purple().italic()
          );
        }
      },

      None => {
        log::warn!(
          "cannot load JSON file {} because it doesn’t have a sub extension",
          path.display().to_string().purple().italic()
        );
      }
    }
  }

  fn extract_sub_extension(path: &Path) -> Option<&str> {
    path.file_stem().and_then(OsStr::to_str).and_then(|name| {
      let mut components = name.rsplit('.');
      let sub_ext = components.next()?;

      // check if we have at least two components in the name (foo.sub_ext)
      if components.next().is_some() {
        Some(sub_ext)
      } else {
        None
      }
    })
  }

  fn load_parameters(&mut self, path: &Path) {
    match self::parameter::Parameter::load_from_file(path) {
      Ok(params) => {
        let path = path.display().to_string().purple().italic();
        log::info!("{} parameters at {}", "loaded".green().bold(), path);

        // check each parameter and create handle if not already existing; update otherwise
        for (name, param) in params {
          log::debug!("  found parameter {}: {:?}", name.purple().italic(), param);
          self.resources.wrap(Entity::Parameter(param), name);
        }
      }

      Err(err) => {
        log::error!(
          "cannot load {} {}: {}",
          "parameters".yellow().italic(),
          path.display().to_string().purple().italic(),
          err,
        );
      }
    }
  }
}

impl System for EntitySystem {
  type Addr = Addr<EntityMsg>;

  fn system_addr(&self) -> Addr<EntityMsg> {
    self.addr.clone()
  }

  fn startup(self) {
    // move into a thread for greater good
    let _ = thread::spawn(move || {
      self.start();
    });
  }
}

impl Publisher<EntityEvent> for EntitySystem {
  fn subscribe(&mut self, subscriber: impl Recipient<EntityEvent> + 'static) {
    self.subscribers.push(Box::new(subscriber));
  }

  fn publish(&self, event: EntityEvent) {
    for sub in &self.subscribers {
      sub.send_msg(event.clone()).unwrap();
    }
  }
}
