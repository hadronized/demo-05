//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod decoder;
pub mod default_decoders;
pub mod material;
pub mod mesh;
pub mod parameter;
pub mod shader;

use self::{parameter::Parameter, shader::Shader};
use crate::{
  entity::decoder::HasDecoder,
  proto::Kill,
  runtime::RuntimeMsg,
  system::{
    resource::Handle, resource::ResourceManager, system_init, Addr, MsgQueue, Publisher, Recipient,
    System, SystemUID,
  },
};
use colored::Colorize as _;
use mesh::Mesh;
use std::{
  ffi::OsStr,
  fs::read_dir,
  marker::PhantomData,
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
  Parameter(Arc<Parameter>),
  /// A [`Shader`].
  Shader(Arc<Shader>),
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
pub struct EntitySystem<Decoders = default_decoders::Decoders> {
  uid: SystemUID,
  runtime_addr: Addr<RuntimeMsg>,
  /// Directory where all scarce resources this entity system knows about live in.
  root_dir: PathBuf,
  resources: ResourceManager<Entity>,
  addr: Addr<EntityMsg>,
  msg_queue: MsgQueue<EntityMsg>,
  publisher: EntityPublisher,
  _phantom: PhantomData<Decoders>,
}

impl<Decoders> EntitySystem<Decoders>
where
  Decoders: HasDecoder,
{
  /// Create a new [`EntitySystem`].
  pub fn new(runtime_addr: Addr<RuntimeMsg>, uid: SystemUID, root_dir: impl Into<PathBuf>) -> Self {
    let (addr, msg_queue) = system_init(uid);
    let root_dir = root_dir.into();

    Self {
      uid,
      runtime_addr,
      root_dir: root_dir.clone(),
      resources: ResourceManager::new(root_dir),
      addr,
      msg_queue,
      publisher: EntityPublisher::new(),
      _phantom: PhantomData,
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
            let sub_ext = Self::extract_sub_extension(&path).unwrap_or("");
            self.extension_based_dispatch(ext, sub_ext, &path);
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
  fn extension_based_dispatch(&mut self, ext: &str, sub_ext: &str, path: &Path) {
    if !Decoders::load_from_file(&mut self.resources, &mut self.publisher, ext, sub_ext, path) {
      if sub_ext.is_empty() {
        log::warn!(
          "unknown extension {} for path {}",
          ext.yellow().italic(),
          path.display().to_string().purple().italic(),
        );
      } else {
        log::warn!(
          "unknown extension {}.{} for path {}",
          sub_ext.yellow().italic(),
          ext.yellow().italic(),
          path.display().to_string().purple().italic(),
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
}

impl<Decoders> System for EntitySystem<Decoders>
where
  Decoders: 'static + Send + HasDecoder,
{
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

impl<Decoders> Publisher<EntityEvent> for EntitySystem<Decoders> {
  fn subscribe(&mut self, subscriber: impl Recipient<EntityEvent> + 'static) {
    self.publisher.subscribe(subscriber)
  }

  fn publish(&self, event: EntityEvent) {
    self.publisher.publish(event)
  }
}

/// Publisher of [`EntityEvent`].
pub struct EntityPublisher {
  subscribers: Vec<Box<dyn Recipient<EntityEvent>>>,
}

impl EntityPublisher {
  fn new() -> Self {
    Self {
      subscribers: Vec::new(),
    }
  }
}

impl Publisher<EntityEvent> for EntityPublisher {
  fn subscribe(&mut self, subscriber: impl Recipient<EntityEvent> + 'static) {
    self.subscribers.push(Box::new(subscriber));
  }

  fn publish(&self, event: EntityEvent) {
    for sub in &self.subscribers {
      sub.send_msg(event.clone()).unwrap();
    }
  }
}
