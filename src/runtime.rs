//! The runtime system.

#![deny(missing_docs)]

use crate::system::SystemUID;

/// Runtime message.
#[derive(Debug)]
pub enum RuntimeMsg {
  /// A system has exited.
  SystemExit(SystemUID),
}
