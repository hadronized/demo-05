//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

pub mod mesh;

use std::path::PathBuf;
use system::resource::ResourceManager;

use crate::mesh::Mesh;

/// All possible entities.
#[derive(Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Mesh),
}

/// The [`Entity`] system.
struct EntitySystem {
  /// Directory where all scarce resource this entity system knows about live in.
  root_dir: PathBuf,
  resources: ResourceManager<Entity>,
}
