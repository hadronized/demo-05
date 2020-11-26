//! Shader entities.

use crate::{
  entity::{
    decoder::{Decoder, DecodingMetadata},
    Entity, EntityEvent,
  },
  system::{resource::ResourceManager, Publisher},
};
use colored::Colorize as _;
use serde::{Deserialize, Serialize};
use std::{error, fmt, fs, io, path::Path, path::PathBuf, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Shader {
  name: String,
  vert_shader: String,
  tess_ctrl_shader: String,
  tess_eval_shader: String,
  geo_shader: String,
  frag_shader: String,
}

/// Paths for each shader stages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ShaderInfo {
  name: String,
  #[serde(rename = "vertex-shader")]
  vert_shader: PathBuf,
  #[serde(rename = "tessellation-control-shader", default)]
  tess_ctrl_shader: PathBuf,
  #[serde(rename = "tessellation-evaluation-shader", default)]
  tess_eval_shader: PathBuf,
  #[serde(rename = "geometry-shader", default)]
  geo_shader: PathBuf,
  #[serde(rename = "fragment-shader")]
  frag_shader: PathBuf,
}

impl Shader {
  /// Load a shader from a given `path`.
  ///
  /// The `prefix` argument allows to automatically insert a prefix path if `path` starts with `'/'`.
  pub fn load_from_file(
    resources: &mut ResourceManager<Entity>,
    path: impl AsRef<Path>,
  ) -> Result<(Self, ShaderInfo), ShaderError> {
    let path = path.as_ref();

    log::debug!(
      "loading {} {}",
      "shader".yellow().italic(),
      path.display().to_string().purple().italic()
    );

    let content = fs::read_to_string(path)?;
    let shader_info: ShaderInfo = serde_json::from_str(&content)?;

    // vertex shader
    let vert_shader;
    if shader_info.vert_shader.is_file() {
      vert_shader =
        fs::read_to_string(resources.resource_to_relative_path(path, &shader_info.vert_shader))?;
    } else {
      vert_shader = String::new();
    }

    // tessellation control shader
    let tess_ctrl_shader;
    if shader_info.tess_ctrl_shader.is_file() {
      tess_ctrl_shader = fs::read_to_string(
        resources.resource_to_relative_path(path, &shader_info.tess_ctrl_shader),
      )?;
    } else {
      tess_ctrl_shader = String::new();
    }

    // tessellation evaluation shader
    let tess_eval_shader;
    if shader_info.tess_eval_shader.is_file() {
      tess_eval_shader = fs::read_to_string(
        resources.resource_to_relative_path(path, &shader_info.tess_eval_shader),
      )?;
    } else {
      tess_eval_shader = String::new();
    }

    // geometry shader
    let geo_shader;
    if shader_info.geo_shader.is_file() {
      geo_shader =
        fs::read_to_string(resources.resource_to_relative_path(path, &shader_info.geo_shader))?;
    } else {
      geo_shader = String::new();
    }

    // fragment shader
    let frag_shader;
    if shader_info.frag_shader.is_file() {
      frag_shader =
        fs::read_to_string(resources.resource_to_relative_path(path, &shader_info.frag_shader))?;
    } else {
      frag_shader = String::new();
    }

    let shader = Shader {
      name: shader_info.name.clone(),
      vert_shader: vert_shader.clone(),
      tess_ctrl_shader: tess_ctrl_shader.clone(),
      tess_eval_shader: tess_eval_shader.clone(),
      geo_shader: geo_shader.clone(),
      frag_shader: frag_shader.clone(),
    };

    Ok((shader, shader_info))
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

    match Shader::load_from_file(resources, path) {
      Ok((shader, info)) => {
        let entity = Entity::Shader(Arc::new(shader));
        let mut deps = vec![info.vert_shader, info.frag_shader];

        if info.tess_ctrl_shader.is_file() {
          deps.push(info.tess_ctrl_shader);
        }

        if info.tess_eval_shader.is_file() {
          deps.push(info.tess_eval_shader);
        }

        if info.geo_shader.is_file() {
          deps.push(info.geo_shader);
        }

        let handle = resources.wrap(entity.clone(), info.name, DecodingMetadata::with_deps(deps));

        let event = EntityEvent::Loaded { handle, entity };
        publisher.publish(event);

        Ok(())
      }

      Err(err) => Err(err),
    }
  }
}
