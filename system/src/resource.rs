//! Resources handled by a system.
//!
//! External systems might want to talk about the same resource. However, systems will represent resources with
//! different encoding / format / binary form. The [`Handle`] type is designed to be shared between systems in a way
//! that doesn’t leak the internal representation of the resource. Each handle is unique on the whole graph of
//! system, which allows them to know which handle references which local / stateful resources they are handling.

use std::{cmp::Ordering, collections::HashMap, marker::PhantomData};

/// Simple handle systems can talk about.
///
/// It should fit most needs.
#[derive(Debug)]
pub struct Handle<T> {
  id: u32,
  _phantom: PhantomData<*const T>,
}

unsafe impl<T> Send for Handle<T> {}

impl<T> Handle<T> {
  fn copy(&self) -> Self {
    Handle {
      id: self.id,
      _phantom: PhantomData,
    }
  }
}

impl<T> Eq for Handle<T> {}

impl<T> Ord for Handle<T> {
  fn cmp(&self, rhs: &Self) -> Ordering {
    self.id.cmp(&rhs.id)
  }
}

impl<T> PartialEq for Handle<T> {
  fn eq(&self, rhs: &Self) -> bool {
    self.id.eq(&rhs.id)
  }
}

impl<T> PartialOrd for Handle<T> {
  fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
    self.id.partial_cmp(&rhs.id)
  }
}

impl<T> std::hash::Hash for Handle<T> {
  fn hash<H>(&self, state: &mut H)
  where
    H: std::hash::Hasher,
  {
    self.id.hash(state)
  }
}

/// A stateful handle manager, distributing handle.
#[derive(Debug)]
pub struct ResourceManager<T> {
  next_handle: Handle<T>,
  resources: HashMap<Handle<T>, T>,
}

impl<T> ResourceManager<T> {
  /// Create a new [`HandleManager`].
  pub fn new() -> Self {
    Self {
      next_handle: Handle {
        id: 0,
        _phantom: PhantomData,
      },
      resources: HashMap::new(),
    }
  }

  /// Accept a resource in the manager and return a handle to it.
  pub fn wrap(&mut self, resource: T) -> Handle<T> {
    let handle = self.gen_handle();
    let _ = self.resources.insert(handle.copy(), resource);
    handle
  }

  /// Lookup the resource referred to by the input handle.
  pub fn get(&self, handle: Handle<T>) -> Option<&T> {
    self.resources.get(&handle)
  }

  /// Lookup the resource referred to by the input handle.
  pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
    self.resources.get_mut(&handle)
  }

  /// Get rid of a resource.
  pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
    self.resources.remove(&handle)
  }

  /// Generate a new, unique handle.
  fn gen_handle(&mut self) -> Handle<T> {
    let id = self.next_handle.id;
    self.next_handle.id += 1;

    Handle {
      id,
      _phantom: PhantomData,
    }
  }
}
