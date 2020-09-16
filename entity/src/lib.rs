//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod mesh;

use std::path::{Path, PathBuf};
use system::resource::ResourceManager;

use crate::mesh::Mesh;

/// All possible entities.
#[derive(Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Mesh),
}

/// The [`Entity`] system.
pub struct EntitySystem {
  /// Directory where all scarce resources this entity system knows about live in.
  root_dir: PathBuf,
  resources: ResourceManager<Entity>,
}

impl EntitySystem {
  /// Create a new [`EntitySystem`].
  pub fn new(root_dir: impl AsRef<Path>) -> Self {
    Self {
      root_dir: root_dir.as_ref().to_owned(),
      resources: ResourceManager::new(),
    }
  }
}
