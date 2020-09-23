//! Protocols.
//!
//! This crate contains all protocols shared by all systems.

#![deny(missing_docs)]

use system::SystemUID;

/// Runtime message.
#[derive(Debug)]
pub enum RuntimeMsg {
  /// A system has exited.
  SystemExit(SystemUID),
}
