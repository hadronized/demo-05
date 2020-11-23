//! Default decoders.

use crate::entity::{decoder::Tuple, mesh::OBJDecoder, parameter::ParameterDecoder};

pub type Decoders = Tuple<OBJDecoder, ParameterDecoder>;
