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
use colored::Colorize as _;
use glfw::{Action, Context as _, Key, WindowEvent};
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
  surface: GlfwSurface,
  meshes: HashMap<Handle<Entity>, Tess<MeshVertex, MeshIndex>>,
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
    log::info!("accepting entity for handle {}", handle);

    match entity {
      Entity::Mesh(mesh) => self.accept_mesh(handle, mesh),
    }
  }

  /// Accept a mesh.
  fn accept_mesh(&mut self, handle: Handle<Entity>, mesh: Arc<Mesh>) {
    log::debug!("building GPU tessellation for handle {}", handle);
  }
}

impl System for GraphicsSystem {
  type Addr = Addr<GraphicsMsg>;

  fn system_addr(&self) -> Self::Addr {
    self.addr.clone()
  }

  fn startup(mut self) {
    log::info!("starting");

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
