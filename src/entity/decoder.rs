//! Resource decoders.

use crate::{
  entity::{Entity, EntityEvent},
  system::{resource::ResourceManager, Publisher},
};
use colored::Colorize as _;
use std::{collections::HashSet, error::Error, path::Path, path::PathBuf};

/// Resource decoder.
pub trait Decoder: Sized {
  /// File extension this decoder accepts.
  const EXT: &'static str;

  /// File sub extension this decoder accepts.
  ///
  /// If sub extensions don’t make sense for your implementor, simply use the empty string `""`.
  const SUB_EXT: &'static str;

  /// Decoder error.
  type Err: Error;

  /// Load a resource from a path.
  fn load_from_file(
    resources: &mut ResourceManager<Entity>,
    publisher: &mut impl Publisher<EntityEvent>,
    path: impl AsRef<Path>,
  ) -> Result<(), Self::Err>;
}

/// Information passed around after decoding to trace dependencies and other kind of data.
#[derive(Debug)]
pub struct DecodingMetadata {
  pub path_deps: HashSet<PathBuf>,
}

impl DecodingMetadata {
  pub fn new() -> Self {
    Self {
      path_deps: HashSet::new(),
    }
  }

  /// Add a dependency.
  ///
  /// If the dependency was not already present, return `true`, `false otherwise`.
  pub fn add_dep(&mut self, path: impl Into<PathBuf>) -> bool {
    self.path_deps.insert(path.into())
  }

  /// Shortcut to create a [`DecodingMetadata`] containing dependencies for only one resource.
  pub fn with_deps(paths: impl IntoIterator<Item = PathBuf>) -> Self {
    let mut dmd = Self::new();

    for path in paths {
      let _ = dmd.add_dep(path);
    }

    dmd
  }
}

/// Type that contains decoders.
pub trait HasDecoder {
  fn load_from_file(
    resources: &mut ResourceManager<Entity>,
    publisher: &mut impl Publisher<EntityEvent>,
    ext: impl AsRef<str>,
    sub_ext: impl AsRef<str>,
    path: impl AsRef<Path>,
  ) -> bool;
}

impl<D> HasDecoder for D
where
  D: Decoder,
{
  fn load_from_file(
    resources: &mut ResourceManager<Entity>,
    publisher: &mut impl Publisher<EntityEvent>,
    ext: impl AsRef<str>,
    sub_ext: impl AsRef<str>,
    path: impl AsRef<Path>,
  ) -> bool {
    let path = path.as_ref();
    let was_active = ext.as_ref() == D::EXT && sub_ext.as_ref() == D::SUB_EXT;

    if was_active {
      if let Err(err) = <D as Decoder>::load_from_file(resources, publisher, path) {
        log::error!(
          "cannot load {} {}: {}",
          "obj".yellow().italic(),
          path.display().to_string().purple().italic(),
          err,
        );
      }
    }

    was_active
  }
}

macro_rules! impl_has_decoder_tuples {
  ($t:tt) => {};

  ($first:tt , $($t:tt),*) => {
    impl_has_decoder_tuple!($first, $($t),*);
    impl_has_decoder_tuples!($($t),*);
  };
}

macro_rules! impl_has_decoder_tuple {
  ($($t:tt),*) => {
    impl<$($t),*> HasDecoder for ( $($t),* )
    where
      $($t: HasDecoder),*
    {
      fn load_from_file(
        resources: &mut ResourceManager<Entity>,
        publisher: &mut impl Publisher<EntityEvent>,
        ext: impl AsRef<str>,
        sub_ext: impl AsRef<str>,
        path: impl AsRef<Path>,
      ) -> bool {
        let ext = ext.as_ref();
        let sub_ext = sub_ext.as_ref();
        let path = path.as_ref();

        $( $t::load_from_file(resources, publisher, ext, sub_ext, path) || )* false
      }
    }
  }
}

impl_has_decoder_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
