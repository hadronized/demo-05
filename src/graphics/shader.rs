//! Shaders on the GPU.

use glsl::{
  syntax::{
    FullySpecifiedType, ShaderStage, SingleDeclaration, StorageQualifier, TypeQualifier,
    TypeQualifierSpec, TypeSpecifier, TypeSpecifierNonArray,
  },
  visitor::{Host as _, Visit, Visitor},
};
use luminance::shader::{UniformBuilder, UniformWarning};
use luminance_front::{
  shader::{Uniform, UniformInterface},
  Backend,
};
use std::collections::HashMap;

/// Dynamic uniform interface for shaders.
///
/// Shaders compiled and introduced at runtime cannot have a static uniform interface set up front — or it would
/// seriously affect the options people have to customize them. However, they still need to be able to have a form of
/// customization. This type represents an interface whose uniforms are detected by GLSL introspection, and available
/// via a name mapping (a bit like a hash-map). The types are checked at runtime.
#[derive(Debug)]
pub struct DynamicUniformInterface {
  uniforms: HashMap<String, DynamicUniform>,
}

impl DynamicUniformInterface {
  /// Check and query a `[DynamicUniform]`.
  pub fn get(&self, name: impl AsRef<str>) -> Option<&DynamicUniform> {
    self.uniforms.get(name.as_ref())
  }
}

/// Uniform which type is set at runtime.
#[derive(Debug)]
pub enum DynamicUniform {
  // 1D
  Bool(Uniform<bool>),
  Int(Uniform<i32>),
  UInt(Uniform<u32>),
  Float(Uniform<f32>),
  // 2D
  Bool2(Uniform<[bool; 2]>),
  Int2(Uniform<[i32; 2]>),
  UInt2(Uniform<[u32; 2]>),
  Float2(Uniform<[f32; 2]>),
  // 3D
  Bool3(Uniform<[bool; 3]>),
  Int3(Uniform<[i32; 3]>),
  UInt3(Uniform<[u32; 3]>),
  Float3(Uniform<[f32; 3]>),
  // 4D
  Bool4(Uniform<[bool; 4]>),
  Int4(Uniform<[i32; 4]>),
  UInt4(Uniform<[u32; 4]>),
  Float4(Uniform<[f32; 4]>),
}

#[derive(Debug)]
pub struct ShaderASTs<'a> {
  pub vert_ast: &'a ShaderStage,
  pub tess_ctrl_ast: Option<&'a ShaderStage>,
  pub tess_eval_ast: Option<&'a ShaderStage>,
  pub geo_ast: Option<&'a ShaderStage>,
  pub frag_ast: &'a ShaderStage,
}

impl<'b> UniformInterface<Backend, ShaderASTs<'b>> for DynamicUniformInterface {
  fn uniform_interface<'a>(
    builder: &mut UniformBuilder<'a, Backend>,
    asts: &mut ShaderASTs<'b>,
  ) -> Result<Self, UniformWarning> {
    // extract the name + type of all uniforms declared in vertex, tessellation, geometry and fragment shader stages by
    // using a GLSL AST visitor
    let mut extractor = ExtractUniforms::new();

    asts.vert_ast.visit(&mut extractor);

    if let (Some(ctrl), Some(eval)) = (&mut asts.tess_ctrl_ast, &mut asts.tess_eval_ast) {
      ctrl.visit(&mut extractor);
      eval.visit(&mut extractor);
    }

    if let Some(ref mut geo) = asts.geo_ast {
      geo.visit(&mut extractor);
    }

    asts.frag_ast.visit(&mut extractor);

    Ok(extractor.extract_uniforms(builder))
  }
}

/// Extract uniforms from GLSL ASTs.
///
/// This type allows to get information about uniforms (name and types) by traversing GLSL ASTs.
struct ExtractUniforms {
  uniforms: HashMap<String, TypeSpecifierNonArray>,
  errors: Vec<(String, TypeSpecifierNonArray)>,
}

impl ExtractUniforms {
  fn new() -> Self {
    Self {
      uniforms: HashMap::new(),
      errors: Vec::new(),
    }
  }

  fn extract_uniforms(self, builder: &mut UniformBuilder<Backend>) -> DynamicUniformInterface {
    let uniforms = self
      .uniforms
      .into_iter()
      .filter_map(|(name, ty)| {
        let uniform = match ty {
          TypeSpecifierNonArray::Bool => DynamicUniform::Bool(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::Int => DynamicUniform::Int(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::UInt => DynamicUniform::UInt(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::Float => DynamicUniform::Float(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::Vec2 => DynamicUniform::Float2(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::Vec3 => DynamicUniform::Float3(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::Vec4 => DynamicUniform::Float4(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::BVec2 => DynamicUniform::Bool2(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::BVec3 => DynamicUniform::Bool3(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::BVec4 => DynamicUniform::Bool4(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::IVec2 => DynamicUniform::Int2(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::IVec3 => DynamicUniform::Int3(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::IVec4 => DynamicUniform::Int4(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::UVec2 => DynamicUniform::UInt2(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::UVec3 => DynamicUniform::UInt3(builder.ask_or_unbound(&name)),
          TypeSpecifierNonArray::UVec4 => DynamicUniform::UInt4(builder.ask_or_unbound(&name)),
          _ => {
            log::warn!(
              "uniform {} has a type ({:?}) that is not currently supported; ignoring",
              name,
              ty
            );

            return None;
          }
        };

        log::trace!("found uniform {} of type {:?}", name, ty);

        Some((name, uniform))
      })
      .collect();

    for (name, ty) in self.errors {
      log::warn!(
        "dynamic uniform interface error for uniform {}, which type is {:?}",
        name,
        ty
      )
    }

    DynamicUniformInterface { uniforms }
  }
}

impl Visitor for ExtractUniforms {
  fn visit_single_declaration(&mut self, sd: &SingleDeclaration) -> Visit {
    match sd {
      SingleDeclaration {
        ty:
          FullySpecifiedType {
            qualifier: Some(TypeQualifier { qualifiers: quals }),
            ty: TypeSpecifier {
              ty,
              array_specifier: None,
            },
          },
        name: Some(name),
        array_specifier: None,
        initializer: None,
      } if quals.0.len() == 1 => {
        if let TypeQualifierSpec::Storage(StorageQualifier::Uniform) = quals.0[0] {
          // we matched a uniform declaration
          let name = name.0.clone();
          let ty = ty.clone();

          if self.uniforms.contains_key(&name) {
            self.errors.push((name, ty));
          } else {
            self.uniforms.insert(name, ty);
          }
        }
      }

      _ => (),
    }

    Visit::Parent
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use glsl::parser::Parse as _;

  #[test]
  fn uniform_extractor() {
    let vs = r#"
      uniform float x;
      uniform ivec2 y;
      uniform int z;

      void main() {}
      "#;
    let mut ast = glsl::syntax::ShaderStage::parse(vs).unwrap();
    let mut extractor = ExtractUniforms::new();

    ast.visit(&mut extractor);

    assert_eq!(
      extractor.uniforms.get("x"),
      Some(&TypeSpecifierNonArray::Float)
    );
    assert_eq!(
      extractor.uniforms.get("y"),
      Some(&TypeSpecifierNonArray::IVec2)
    );
    assert_eq!(
      extractor.uniforms.get("z"),
      Some(&TypeSpecifierNonArray::Int)
    );
  }
}
