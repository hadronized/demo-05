//! Graphics system.
//!
//! This system is responsible in all the rendering operations.

use crate::{
  proto::Kill,
  runtime::RuntimeMsg,
  system::{system_init, Addr, MsgQueue, System, SystemUID},
};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use luminance_windowing::WindowOpt;
use std::fmt;

const TITLE: &str = "Spectra";

#[derive(Clone, Debug)]
pub enum GraphicsMsg {
  /// Kill message.
  Kill,
}

impl From<Kill> for GraphicsMsg {
  fn from(_: Kill) -> Self {
    Self::Kill
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
}

impl GraphicsSystem {
  pub fn new(
    runtime_addr: Addr<RuntimeMsg>,
    uid: SystemUID,
    win_opt: WindowOpt,
  ) -> Result<Self, GraphicsSystemError> {
    let (addr, msg_queue) = system_init(uid);
    let surface = GlfwSurface::new_gl33(TITLE, win_opt)?;

    Ok(Self {
      uid,
      runtime_addr,
      addr,
      msg_queue,
      surface,
    })
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
