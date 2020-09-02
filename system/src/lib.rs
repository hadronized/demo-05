//! The System crate.
//!
//! This crate provides a very simple mechanism to create _systems_, which can send messages to each other, spawn new
//! systems and perform local state mutation and I/O.

pub mod resource;

trait System {
  type Addr;

  /// Return a [`System::Addr`], representing the current system, that can be shared with other systems if they want
  /// to reply to messages this system send.
  fn system_handle(&self) -> Self::Addr;
}
