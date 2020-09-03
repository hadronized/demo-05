//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

#![feature(bool_to_option)]

mod mesh;

use crate::mesh::Mesh;

/// All possible entities.
#[derive(Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Mesh),
}
