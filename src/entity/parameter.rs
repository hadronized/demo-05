//! Varying parameters.
//!
//! Parameters are generic runtime objects that are used to customize the behavior of various different systems.
//! Parameters come in several flavors:
//!
//! - **Variable parameters**: those are values specified directly from a configuration file or set manually, for
//!   instance. They allow an indirection so that they can be shared and propagated to whichever needs them.
//! - **Animation parameters**: when parameters need to change over time, it’s great to be able to specify the
//!   behavior of the parameter as a function of time. Those parameters implement different kind of animation
//!   parameters, depending on your need (constant, linear, cosine, Bézier, etc.).

use crate::entity::{decoder::Decoder, Entity, EntityEvent};
use colored::Colorize as _;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, error, fmt, fs, io, path::Path, sync::Arc};

#[derive(Debug)]
pub enum ParameterError {
  FileError(io::Error),
  JSONError(serde_json::Error),
  NoData,
}

impl fmt::Display for ParameterError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      ParameterError::FileError(ref e) => write!(f, "file error: {}", e),
      ParameterError::JSONError(ref e) => write!(f, "JSON error: {}", e),
      ParameterError::NoData => f.write_str("no parameter detected"),
    }
  }
}

impl error::Error for ParameterError {}

impl From<io::Error> for ParameterError {
  fn from(err: io::Error) -> Self {
    Self::FileError(err)
  }
}

impl From<serde_json::Error> for ParameterError {
  fn from(err: serde_json::Error) -> Self {
    Self::JSONError(err)
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Parameter {
  #[serde(rename = "const")]
  Constant(Constant),
}

impl Parameter {
  pub fn load_from_file(
    path: impl AsRef<Path>,
  ) -> Result<HashMap<String, Parameter>, ParameterError> {
    let path = path.as_ref();

    log::debug!(
      "loading {} {}",
      "parameters".yellow().italic(),
      path.display().to_string().purple().italic()
    );

    let content = fs::read_to_string(path)?;
    let parameters: HashMap<_, _> = serde_json::from_str(&content)?;

    if parameters.is_empty() {
      Err(ParameterError::NoData)
    } else {
      Ok(parameters)
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParameterDecoder;

impl Decoder for ParameterDecoder {
  const EXT: &'static str = "json";

  const SUB_EXT: &'static str = "param";

  type Err = ParameterError;

  fn load_from_file(
    resources: &mut crate::system::resource::ResourceManager<super::Entity>,
    publisher: &mut impl crate::system::Publisher<super::EntityEvent>,
    path: impl AsRef<Path>,
  ) -> Result<(), Self::Err> {
    let path = path.as_ref();

    match Parameter::load_from_file(path) {
      Ok(params) => {
        let path = path.display().to_string().purple().italic();
        log::info!("{} parameters at {}", "loaded".green().bold(), path);

        // check each parameter and create handle if not already existing; update otherwise
        for (name, param) in params {
          log::debug!("  found parameter {}: {:?}", name.purple().italic(), param);
          let entity = Entity::Parameter(Arc::new(param));
          let handle = resources.wrap(entity.clone(), name);

          let event = EntityEvent::Loaded { handle, entity };
          publisher.publish(event);
        }

        Ok(())
      }

      Err(err) => Err(err),
    }
  }
}

macro_rules! uint_serde_override {
  ($t:tt, $r:tt) => {
    #[derive(Debug, Deserialize)]
    struct $t {
      unsigned: $r,
    }

    impl $t {
      fn deserialize_override<'d, D>(deserializer: D) -> Result<$r, D::Error>
      where
        D: Deserializer<'d>,
      {
        let Self { unsigned } = Self::deserialize(deserializer)?;
        Ok(unsigned)
      }
    }
  };
}

uint_serde_override!(UInt, u32);
uint_serde_override!(UInt2, [u32; 2]);
uint_serde_override!(UInt3, [u32; 3]);
uint_serde_override!(UInt4, [u32; 4]);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Constant {
  // 1D
  Bool(bool),
  Int(i32),
  #[serde(deserialize_with = "UInt::deserialize_override")]
  UInt(u32),
  Float(f32),
  // 2D
  Bool2([bool; 2]),
  Int2([i32; 2]),
  #[serde(deserialize_with = "UInt2::deserialize_override")]
  UInt2([u32; 2]),
  Float2([f32; 2]),
  // 3D
  Bool3([bool; 3]),
  Int3([i32; 3]),
  #[serde(deserialize_with = "UInt3::deserialize_override")]
  UInt3([u32; 3]),
  Float3([f32; 3]),
  // 4D
  Bool4([bool; 4]),
  Int4([i32; 4]),
  #[serde(deserialize_with = "UInt4::deserialize_override")]
  UInt4([u32; 4]),
  Float4([f32; 4]),
}

// 1D
impl From<bool> for Constant {
  fn from(a: bool) -> Self {
    Self::Bool(a)
  }
}

impl From<i32> for Constant {
  fn from(a: i32) -> Self {
    Self::Int(a)
  }
}

impl From<u32> for Constant {
  fn from(a: u32) -> Self {
    Self::UInt(a)
  }
}

impl From<f32> for Constant {
  fn from(a: f32) -> Self {
    Self::Float(a)
  }
}

// 2D
impl From<[bool; 2]> for Constant {
  fn from(a: [bool; 2]) -> Self {
    Self::Bool2(a)
  }
}

impl From<[i32; 2]> for Constant {
  fn from(a: [i32; 2]) -> Self {
    Self::Int2(a)
  }
}

impl From<[u32; 2]> for Constant {
  fn from(a: [u32; 2]) -> Self {
    Self::UInt2(a)
  }
}

impl From<[f32; 2]> for Constant {
  fn from(a: [f32; 2]) -> Self {
    Self::Float2(a)
  }
}

// 3D
impl From<[bool; 3]> for Constant {
  fn from(a: [bool; 3]) -> Self {
    Self::Bool3(a)
  }
}

impl From<[i32; 3]> for Constant {
  fn from(a: [i32; 3]) -> Self {
    Self::Int3(a)
  }
}

impl From<[u32; 3]> for Constant {
  fn from(a: [u32; 3]) -> Self {
    Self::UInt3(a)
  }
}

impl From<[f32; 3]> for Constant {
  fn from(a: [f32; 3]) -> Self {
    Self::Float3(a)
  }
}

// 4D
impl From<[bool; 4]> for Constant {
  fn from(a: [bool; 4]) -> Self {
    Self::Bool4(a)
  }
}

impl From<[i32; 4]> for Constant {
  fn from(a: [i32; 4]) -> Self {
    Self::Int4(a)
  }
}

impl From<[u32; 4]> for Constant {
  fn from(a: [u32; 4]) -> Self {
    Self::UInt4(a)
  }
}

impl From<[f32; 4]> for Constant {
  fn from(a: [f32; 4]) -> Self {
    Self::Float4(a)
  }
}
