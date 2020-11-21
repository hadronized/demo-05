//! Resources handled by a system.
//!
//! External systems might want to talk about the same resource. However, systems will represent resources with
//! different encoding / format / binary form. The [`Handle`] type is designed to be shared between systems in a way
//! that doesn’t leak the internal representation of the resource. Each handle is unique on the whole graph of
//! system, which allows them to know which handle references which local / stateful resources they are handling.

use colored::Colorize as _;
use std::{cmp::Ordering, collections::HashMap, fmt, marker::PhantomData};

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

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
  fn clone(&self) -> Self {
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

impl<T> fmt::Display for Handle<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.id.to_string().cyan().bold())
  }
}

/// A stateful handle manager, distributing handle.
#[derive(Debug)]
pub struct ResourceManager<T> {
  next_handle: Handle<T>,
  resources: HashMap<Handle<T>, T>,
  translations: HashMap<String, Handle<T>>,
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
      translations: HashMap::new(),
    }
  }

  /// Accept a resource in the manager and return a handle to it.
  ///
  /// The `name` parameter refers to the identifier external systems might give to this resource. You cannot use it to
  /// ask this resource back, but you can get a [`Handle`] from an identifier later. See the [`ResourceManager::translate`] method for further information.
  pub fn wrap(&mut self, resource: T, name: impl AsRef<str>) -> Handle<T> {
    let name = name.as_ref();

    match self.ask(name) {
      Some(handle) => {
        // the resource already exists; let’s just replace it
        log::debug!(
          "replacing resource {} {}",
          name.blue().bold(),
          handle.to_string().green().bold()
        );
        self.resources.insert(handle, resource);

        handle
      }

      None => {
        // this is the first time we see this resource; wrap it up
        let handle = self.gen_handle();
        let name = name.to_owned();
        log::debug!(
          "wrapping resource {} with handle {}",
          name.blue().bold(),
          handle.to_string().green().bold()
        );

        let _ = self.resources.insert(handle.copy(), resource);
        let _ = self.translations.insert(name, handle.clone());

        handle
      }
    }
  }

  /// Translate a resource name into a handle.
  ///
  /// This function allows to check whether a resource is already registered and eventually modify it.
  pub fn ask(&self, name: impl AsRef<str>) -> Option<Handle<T>> {
    self.translations.get(name.as_ref()).copied()
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
