//! The entity system.
//!
//! This system is responsible in loading, streaming and watching assets, also known as _entities_. Entities are
//! independent objects identified by a unique identifier.

use luminance::tess::Mode;
use luminance_derive::{Semantics, Vertex};

/// Unique identifier for entities.
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityID(u32);

/// All possible entities.
#[derive(Debug)]
pub enum Entity {
  /// A [`Mesh`].
  Mesh(Mesh),
}

/// A mesh.
///
/// A mesh consists of a set of vertices, indices and a primitive mode.
#[derive(Debug)]
pub struct Mesh {
  vertices: Vec<MeshVertex>,
  indices: Vec<MeshIndex>,
  mode: Mode,
}

/// Vertex index used in [`Mesh`] to create primitive by connecting vertices.
pub type MeshIndex = u32;

/// Vertex semantics used in [`Mesh`].
#[derive(Clone, Copy, Debug, Semantics)]
pub enum VertexSemantics {
  /// Vertex position.
  #[sem(name = "pos", repr = "[f32; 3]", wrapper = "Pos")]
  Position,

  /// Vertex normal.
  #[sem(name = "nor", repr = "[f32; 3]", wrapper = "Nor")]
  Normal,
}

/// Vertex type used in [`Mesh`].
#[vertex(sem = "VertexSemantics")]
#[derive(Clone, Copy, Debug, Vertex)]
pub struct MeshVertex {
  /// Position of the vertex.
  pos: Pos,

  /// Normal of the vertex.
  nor: Nor,
}
