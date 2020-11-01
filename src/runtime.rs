//! The runtime system.

#![deny(missing_docs)]

use crate::system::SystemUID;

/// Runtime message.
#[derive(Clone, Debug)]
pub enum RuntimeMsg {
  /// A system has exited.
  SystemExit(SystemUID),
}
