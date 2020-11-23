//! Default decoders.

use crate::entity::{mesh::OBJDecoder, parameter::ParameterDecoder};

pub type Decoders = (OBJDecoder, ParameterDecoder);
