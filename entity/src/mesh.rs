//! Mesh related code.

use luminance::tess::Mode;
use luminance_derive::{Semantics, Vertex};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use wavefront_obj::obj;

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

/// A mesh.
///
/// A mesh consists of a set of vertices, indices and a primitive mode.
#[derive(Debug)]
pub struct Mesh {
  vertices: Vec<MeshVertex>,
  indices: Vec<MeshIndex>,
  mode: Mode,
}

impl Mesh {
  fn validate_path(path: &Path) -> Result<(), MeshLoadingError> {
    log::info!("loading mesh at {}", path.display());

    if !path.is_file() {
      Err(MeshLoadingError::cannot_read_path(path, "not a file"))
    } else {
      Ok(())
    }
  }

  pub fn load_from_path(path: &Path) -> Result<Self, MeshLoadingError> {
    Self::validate_path(path)?;

    // read the content of the path at once (no streaming)
    log::debug!("  opening mesh {}", path.display());
    let file_content = fs::read_to_string(path)
      .map_err(|e| MeshLoadingError::cannot_read_path(path, e.to_string()))?;

    let obj_set =
      obj::parse(file_content).map_err(|e| MeshLoadingError::cannot_parse(path, e.to_string()))?;

    Self::traverse_obj_set(obj_set)
  }

  fn traverse_obj_set(obj_set: obj::ObjSet) -> Result<Self, MeshLoadingError> {
    let objects = obj_set.objects;
    if objects.len() != 1 {
      return Err(MeshLoadingError::too_many_objects(objects.len()));
    }

    let object = objects.into_iter().next().unwrap();
    if object.geometry.len() != 1 {
      return Err(MeshLoadingError::too_many_geometries(object.geometry.len()));
    }

    Self::traverse_object(object)
  }

  fn traverse_object(object: obj::Object) -> Result<Self, MeshLoadingError> {
    let geometry = object.geometry.into_iter().next().unwrap();

    log::info!("  loading object {}", object.name);
    log::info!("    {} vertices", object.vertices.len());
    log::info!("    {} shapes", geometry.shapes.len());

    Self::traverse_geometry(object.vertices, object.normals, geometry)
  }

  fn traverse_geometry(
    obj_vertices: Vec<obj::Vertex>,
    obj_normals: Vec<obj::Normal>,
    geometry: obj::Geometry,
  ) -> Result<Self, MeshLoadingError> {
    // build up vertices; for this to work, we remove duplicated vertices by putting them in a
    // map associating the vertex with its ID
    let mut vertex_cache: HashMap<obj::VTNIndex, MeshIndex> = HashMap::new();
    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut indices: Vec<MeshIndex> = Vec::new();

    for shape in geometry.shapes {
      if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
        for key in &[a, b, c] {
          if let Some(vertex_index) = vertex_cache.get(key) {
            indices.push(*vertex_index);
          } else {
            let p = obj_vertices[key.0];
            let n = obj_normals[key.2.ok_or_else(MeshLoadingError::missing_vertex_normal)?];
            let pos = Pos::new([p.x as f32, p.y as f32, p.z as f32]);
            let nor = Nor::new([n.x as f32, n.y as f32, n.z as f32]);
            let vertex = MeshVertex { pos, nor };
            let vertex_index = vertices.len() as MeshIndex;

            vertex_cache.insert(*key, vertex_index);
            vertices.push(vertex);
            indices.push(vertex_index);
          }
        }
      } else {
        return Err(MeshLoadingError::unsupported_primitive_mode());
      }
    }

    let mode = Mode::Triangle;

    Ok(Mesh {
      vertices,
      indices,
      mode,
    })
  }
}

/// Possible errors that can happen while loading a [`Mesh`].
#[derive(Debug)]
#[non_exhaustive]
pub enum MeshLoadingError {
  /// Incorrect path (doesn’t exist / not enough permission to read from it / etc.).
  CannotReadPath {
    path: PathBuf,
    additional_context: String,
  },

  /// Parsing failure while loading a mesh.
  CannotParse { path: PathBuf, reason: String },

  // TODO: this should be supported and generates several Mesh
  // TODO: add the path of the mesh being loaded
  /// Too many objects detected while loading the mesh.
  TooManyObjects(usize),

  /// Too many geometries detected while loading the mesh.
  // TODO: add the path of the mesh being loaded and the object name
  TooManyGeometries(usize),

  /// Insupported primitive mode.
  UnsupportedPrimitiveMode,

  /// A vertex is missing a normal.
  MissingVertexNormal,
}

impl MeshLoadingError {
  fn cannot_read_path(path: impl Into<PathBuf>, additional_context: impl Into<String>) -> Self {
    MeshLoadingError::CannotReadPath {
      path: path.into(),
      additional_context: additional_context.into(),
    }
  }

  fn cannot_parse(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
    MeshLoadingError::CannotParse {
      path: path.into(),
      reason: reason.into(),
    }
  }

  fn too_many_objects(nb: usize) -> Self {
    MeshLoadingError::TooManyObjects(nb)
  }

  fn too_many_geometries(nb: usize) -> Self {
    MeshLoadingError::TooManyGeometries(nb)
  }

  fn unsupported_primitive_mode() -> Self {
    MeshLoadingError::UnsupportedPrimitiveMode
  }

  fn missing_vertex_normal() -> Self {
    MeshLoadingError::MissingVertexNormal
  }
}

impl fmt::Display for MeshLoadingError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      MeshLoadingError::CannotReadPath {
        ref path,
        ref additional_context,
      } => {
        write!(f, "cannot read path {}", path.display())?;

        if !additional_context.is_empty() {
          write!(f, ": {}", additional_context)?;
        }

        Ok(())
      }

      MeshLoadingError::CannotParse {
        ref path,
        ref reason,
      } => write!(f, "cannot parse {}: {}", path.display(), reason),

      MeshLoadingError::TooManyObjects(nb) => write!(f, "too many objects; {}", nb),

      MeshLoadingError::TooManyGeometries(nb) => write!(f, "too many geometries; {}", nb),

      MeshLoadingError::UnsupportedPrimitiveMode => write!(f, "unsupported primitive mode"),

      MeshLoadingError::MissingVertexNormal => write!(f, "a vertex is missing its normal"),
    }
  }
}
