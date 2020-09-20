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
pub trait System<M> {
  /// Get the address of this system.
  fn system_addr(&self) -> Addr<M>;

  /// Send a message to another system.
  fn send_msg<T>(&self, addr: Addr<T>, msg: T) -> Result<(), SystemError> {
    addr.sender.send(msg).map_err(|_| SystemError::CannotSend)
  }

  /// Run the system and return its [`Addr`] so that other systems can use it.
  fn startup(self) -> Addr<M>;
}

/// An address of a [`System`] that allows sending messages of type `T`.
#[derive(Debug)]
pub struct Addr<T> {
  sender: mpsc::Sender<T>,
}

impl<T> Clone for Addr<T> {
  fn clone(&self) -> Self {
    Addr {
      sender: self.sender.clone(),
    }
  }
}

/// Errors that might occur with [`System`] operations.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum SystemError {
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

/// A message queue, which systems can use to see what messages they have received.
#[derive(Debug)]
pub struct MsgQueue<T> {
  receiver: mpsc::Receiver<T>,
}

/// Default implementation of a system initialization procedure.
///
/// When creating a [`System`], the first thing one wants to do is to create all the required state to be able to:
///
/// - Look at received messages.
/// - Present oneself to others by handing out an [`Addr`].
///
/// This method is supposed to be used by systems’ implementations to ease creating the internal state of a system.
pub fn system_init<T>() -> (Addr<T>, MsgQueue<T>) {
  let (sender, receiver) = mpsc::channel();

  (Addr { sender }, MsgQueue { receiver })
}
