//! Default decoders.

use crate::entity::{mesh::OBJDecoder, parameter::ParameterDecoder, shader::JSONShaderDecoder};

pub type Decoders = (OBJDecoder, ParameterDecoder, JSONShaderDecoder);
