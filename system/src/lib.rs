//! The System crate.
//!
//! This crate provides a very simple mechanism to create _systems_, which can send messages to each other, spawn new
//! systems and perform local state mutation and I/O.

use std::fmt;
use std::sync::mpsc;

pub mod resource;

/// Systems.
///
/// A _system_ is a special kind of object that has an _address_ ([`Addr`]) that other systems can use to send messages
/// to.
///
/// A _message_ can be anything, but most of the time, systems will expect a protocol to be implemented when sending
/// messages to efficiently _move_ messages without having to serialize / deserialize them.
trait System<M> {
  /// Get the address of this system.
  fn my_addr(&self) -> Addr<M>;

  /// Send a message to another system.
  fn send_msg<T>(&self, addr: Addr<T>, msg: T) -> Result<(), SystemError> {
    addr.sender.send(msg).map_err(|_| SystemError::CannotSend)
  }
}

/// An address of a [`System`] that allows sending messages of type `T`.
pub struct Addr<T> {
  sender: mpsc::Sender<T>,
}

/// Errors that might occur with [`System`] operations.
#[derive(Debug)]
enum SystemError {
  /// Cannot send a message.
  CannotSend,
}

impl fmt::Display for SystemError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SystemError::CannotSend => write!(f, "cannot send message"),
    }
  }
}
