//! The runtime system.

#![deny(missing_docs)]

use crate::{proto::Kill, system::SystemUID};

/// Runtime message.
#[derive(Clone, Debug)]
pub enum RuntimeMsg {
  /// A system has exited.
  SystemExit(SystemUID),
  /// Kill the runtime system.
  ///
  /// This will kill in cascade all systems created by the runtime system.
  Kill,
}

impl From<Kill> for RuntimeMsg {
  fn from(_: Kill) -> Self {
    RuntimeMsg::Kill
  }
}
