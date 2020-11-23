//! Resource decoders.

use crate::{
  entity::{Entity, EntityEvent},
  system::{resource::ResourceManager, Publisher},
};
use colored::Colorize as _;
use std::{error::Error, marker::PhantomData, path::Path};

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

/// Type-level tuple of decoders.
#[derive(Debug)]
pub struct Tuple<A, B> {
  _phantom: PhantomData<(A, B)>,
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

impl<A, B> HasDecoder for Tuple<A, B>
where
  A: HasDecoder,
  B: HasDecoder,
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

    A::load_from_file(resources, publisher, ext, sub_ext, path)
      || B::load_from_file(resources, publisher, ext, sub_ext, path)
  }
}
