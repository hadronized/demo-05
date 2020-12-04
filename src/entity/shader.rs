//! Shader entities.

use crate::{
  entity::{
    decoder::{Decoder, DecodingMetadata},
    Entity, EntityEvent,
  },
  system::{resource::ResourceManager, Publisher},
};
use colored::Colorize as _;
use glsl::{parser::Parse as _, parser::ParseError, syntax::ShaderStage};
use serde::{Deserialize, Serialize};
use std::{error, fmt, fs, io, path::Path, path::PathBuf, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct Shader {
  pub name: String,
  pub vert_shader: ShaderData,
  pub tess_ctrl_shader: Option<ShaderData>,
  pub tess_eval_shader: Option<ShaderData>,
  pub geo_shader: Option<ShaderData>,
  pub frag_shader: ShaderData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShaderData {
  /// The raw GLSL string.
  pub raw: String,
  /// Parsed GLSL AST.
  pub ast: ShaderStage,
}

impl ShaderData {
  pub fn new(raw: impl Into<String>, ast: ShaderStage) -> Self {
    Self {
      raw: raw.into(),
      ast,
    }
  }
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
    let parent = path.parent().unwrap_or(path);

    log::debug!(
      "loading {} {}",
      "shader".yellow().italic(),
      path.display().to_string().purple().italic()
    );

    let content = fs::read_to_string(path)?;
    let shader_info: ShaderInfo = serde_json::from_str(&content)?;

    // vertex shader
    let vert_path = resources.resource_to_relative_path(parent, &shader_info.vert_shader);
    let vert_shader = if vert_path.is_file() {
      let src = fs::read_to_string(&vert_path)?;
      let ast = ShaderStage::parse(&src)?;
      ShaderData::new(src, ast)
    } else {
      return Err(ShaderError::MissingVertexShader(vert_path));
    };

    // tessellation control shader
    let tess_ctrl_path = resources.resource_to_relative_path(parent, &shader_info.tess_ctrl_shader);
    let tess_ctrl_shader = if tess_ctrl_path.is_file() {
      let src = fs::read_to_string(tess_ctrl_path)?;
      let ast = ShaderStage::parse(&src)?;

      Some(ShaderData::new(src, ast))
    } else {
      None
    };

    // tessellation evaluation shader
    let tess_eval_path = resources.resource_to_relative_path(parent, &shader_info.tess_eval_shader);
    let tess_eval_shader = if tess_eval_path.is_file() {
      let src = fs::read_to_string(tess_eval_path)?;
      let ast = ShaderStage::parse(&src)?;
      Some(ShaderData::new(src, ast))
    } else {
      None
    };

    // geometry shader
    let geo_path = resources.resource_to_relative_path(parent, &shader_info.geo_shader);
    let geo_shader = if geo_path.is_file() {
      let src = fs::read_to_string(geo_path)?;
      let ast = ShaderStage::parse(&src)?;
      Some(ShaderData::new(src, ast))
    } else {
      None
    };

    // fragment shader
    let frag_path = resources.resource_to_relative_path(parent, &shader_info.frag_shader);
    let frag_shader = if frag_path.is_file() {
      let src = fs::read_to_string(&frag_path)?;
      let ast = ShaderStage::parse(&src)?;
      ShaderData::new(src, ast)
    } else {
      return Err(ShaderError::MissingFragmentShader(frag_path));
    };

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
  GLSLError(ParseError),
  MissingVertexShader(PathBuf),
  MissingFragmentShader(PathBuf),
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

impl From<ParseError> for ShaderError {
  fn from(err: ParseError) -> Self {
    Self::GLSLError(err)
  }
}

impl fmt::Display for ShaderError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ShaderError::FileError(ref e) => write!(f, "cannot open file: {}", e),
      ShaderError::JSONError(ref e) => write!(f, "JSON decoding error: {}", e),
      ShaderError::GLSLError(ref e) => write!(f, "GLSL parsing error: {}", e),
      ShaderError::MissingVertexShader(ref path) => {
        write!(f, "missing vertex shader at path {}", path.display())
      }
      ShaderError::MissingFragmentShader(ref path) => {
        write!(f, "missing fragment shader at path {}", path.display())
      }
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

        // create the dependency tracking
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

        let path = path.display().to_string().purple().italic();
        log::info!("{} shader {} at {}", "loaded".green().bold(), handle, path);

        let event = EntityEvent::Loaded { handle, entity };
        publisher.publish(event);

        Ok(())
      }

      Err(err) => Err(err),
    }
  }
}
