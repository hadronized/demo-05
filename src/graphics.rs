//! Graphics system.
//!
//! This system is responsible in all the rendering operations.

use crate::{
  entity::{
    mesh::{Mesh, MeshIndex, MeshVertex},
    Entity, EntityEvent,
  },
  proto::Kill,
  runtime::RuntimeMsg,
  system::{resource::Handle, system_init, Addr, MsgQueue, System, SystemUID},
};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance_front::context::GraphicsContext as _;
use luminance_front::tess::Tess;
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use luminance_windowing::WindowOpt;
use std::{collections::HashMap, fmt, sync::Arc};

const TITLE: &str = "Spectra";

#[derive(Clone, Debug)]
pub enum GraphicsMsg {
  /// Kill message.
  Kill,
  /// Entity event; used to listen to entity system’s notifications.
  EntityEvent(EntityEvent),
}

impl From<Kill> for GraphicsMsg {
  fn from(_: Kill) -> Self {
    Self::Kill
  }
}

impl From<EntityEvent> for GraphicsMsg {
  fn from(a: EntityEvent) -> Self {
    Self::EntityEvent(a)
  }
}

#[derive(Debug)]
pub enum GraphicsSystemError {
  /// Unable to create the surface.
  SurfaceCreationError(GlfwSurfaceError),
}

impl From<GlfwSurfaceError> for GraphicsSystemError {
  fn from(e: GlfwSurfaceError) -> Self {
    GraphicsSystemError::SurfaceCreationError(e)
  }
}

impl fmt::Display for GraphicsSystemError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      GraphicsSystemError::SurfaceCreationError(ref e) => {
        write!(f, "cannot create GLFW surface: {}", e)
      }
    }
  }
}

#[derive(Debug)]
pub struct GraphicsSystem {
  uid: SystemUID,
  runtime_addr: Addr<RuntimeMsg>,
  addr: Addr<GraphicsMsg>,
  msg_queue: MsgQueue<GraphicsMsg>,
  meshes: HashMap<Handle<Entity>, Tess<MeshVertex, MeshIndex>>,
  surface: GlfwSurface,
}

impl Drop for GraphicsSystem {
  fn drop(&mut self) {
    // ensure we have removed the GPU objects prior to anything else; de-allocating the surface while GPU objects still
    // exist is currently not supported and yield a bug
    self.meshes.clear();
  }
}

impl GraphicsSystem {
  pub fn new(
    runtime_addr: Addr<RuntimeMsg>,
    uid: SystemUID,
    win_opt: WindowOpt,
  ) -> Result<Self, GraphicsSystemError> {
    let (addr, msg_queue) = system_init(uid);
    let surface = GlfwSurface::new_gl33(TITLE, win_opt)?;
    let meshes = HashMap::new();

    Ok(Self {
      uid,
      runtime_addr,
      addr,
      msg_queue,
      surface,
      meshes,
    })
  }

  /// React an entity.
  fn accept_entity(&mut self, handle: Handle<Entity>, entity: Entity) {
    log::debug!("accepting entity {}", handle);

    match entity {
      Entity::Mesh(mesh) => self.accept_mesh(handle, mesh),
    }
  }

  /// Accept a mesh.
  fn accept_mesh(&mut self, handle: Handle<Entity>, mesh: Arc<Mesh>) {
    log::info!("accepting mesh {}", handle);
    log::debug!("building GPU tessellation for mesh {}", handle);

    let mesh = &*mesh;
    let tess_res = self
      .surface
      .new_tess()
      .set_vertices(mesh.vertices().clone())
      .set_indices(mesh.indices().clone())
      .set_mode(mesh.mode())
      .build();

    match tess_res {
      Ok(tess) => {
        if self.meshes.insert(handle, tess).is_none() {
          log::info!("mesh {} successfully represented on the GPU", handle);
        } else {
          // the mesh was already present; reload
          log::info!(
            "mesh handle {} was already present and was replaced",
            handle
          );
        }
      }

      Err(err) => {
        log::error!(
          "cannot accept mesh handle {} because the GPU tessellation failed to build; reason: {}",
          handle,
          err
        );
      }
    }
  }
}

impl System for GraphicsSystem {
  type Addr = Addr<GraphicsMsg>;

  fn system_addr(&self) -> Self::Addr {
    self.addr.clone()
  }

  fn startup(mut self) {
    // main loop
    'system: loop {
      // message loop
      while let Some(msg) = self.msg_queue.try_recv() {
        match msg {
          GraphicsMsg::Kill => {
            self
              .runtime_addr
              .send_msg(RuntimeMsg::SystemExit(self.uid))
              .unwrap();
            break 'system;
          }

          GraphicsMsg::EntityEvent(EntityEvent::Loaded { handle, entity }) => {
            self.accept_entity(handle, entity)
          }
        }
      }

      // events
      self.surface.window.glfw.poll_events();
      for (_, event) in glfw::flush_messages(&self.surface.events_rx) {
        if cfg!(feature = "trace-window-events") {
          log::trace!("event: {:?}", event);
        }

        match event {
          WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            self
              .runtime_addr
              .send_msg(RuntimeMsg::SystemExit(self.uid))
              .unwrap();

            // notify the runtime system to kill everybody
            self.runtime_addr.send_msg(Kill).unwrap();
            break 'system;
          }

          _ => (),
        }
      }

      // render
      self.surface.window.swap_buffers();
    }
  }
}
