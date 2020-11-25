//! Shader entities.

use crate::{
  entity::{decoder::Decoder, Entity, EntityEvent},
  system::{resource::ResourceManager, Publisher},
};
use colored::Colorize as _;
use serde::{Deserialize, Serialize};
use std::{error, fmt, fs, io, path::Path, sync::Arc};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Shader {
  name: String,
  #[serde(rename = "vertex-shader")]
  vert_shader: String,
  #[serde(rename = "tessellation-control-shader")]
  tess_ctrl_shader: String,
  #[serde(rename = "tessellation-evaluation-shader")]
  tess_eval_shader: String,
  #[serde(rename = "geometry-shader")]
  geo_shader: String,
  #[serde(rename = "fragment-shader")]
  frag_shader: String,
}

impl Shader {
  pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ShaderError> {
    let path = path.as_ref();

    log::debug!(
      "loading {} {}",
      "shader".yellow().italic(),
      path.display().to_string().purple().italic()
    );

    let content = fs::read_to_string(path)?;
    let shader = serde_json::from_str(&content)?;

    Ok(shader)
  }
}

#[derive(Debug)]
pub enum ShaderError {
  FileError(io::Error),
  JSONError(serde_json::Error),
}

impl From<io::Error> for ShaderError {
  fn from(err: io::Error) -> Self {
    Self::FileError(err)
  }
}

impl From<serde_json::Error> for ShaderError {
  fn from(err: serde_json::Error) -> Self {
    Self::JSONError(err)
  }
}

impl fmt::Display for ShaderError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ShaderError::FileError(ref e) => write!(f, "cannot open file: {}", e),
      ShaderError::JSONError(ref e) => write!(f, "JSON decoding error: {}", e),
    }
  }
}

impl error::Error for ShaderError {}

#[derive(Debug)]
pub struct JSONShaderDecoder;

impl Decoder for JSONShaderDecoder {
  const EXT: &'static str = "json";

  const SUB_EXT: &'static str = "shd";

  type Err = ShaderError;

  fn load_from_file(
    resources: &mut ResourceManager<Entity>,
    publisher: &mut impl Publisher<EntityEvent>,
    path: impl AsRef<Path>,
  ) -> Result<(), Self::Err> {
    let path = path.as_ref();

    match Shader::load_from_file(path) {
      Ok(shader) => {
        let name = shader.name.clone();

        let entity = Entity::Shader(Arc::new(shader));
        let handle = resources.wrap(entity.clone(), name);

        let event = EntityEvent::Loaded { handle, entity };
        publisher.publish(event);

        Ok(())
      }

      Err(err) => Err(err),
    }
  }
}
