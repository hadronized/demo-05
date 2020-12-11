//! Main logic of the demo.

use spectra::{
  proto::Kill,
  runtime::RuntimeMsg,
  system::{system_init, Addr, MsgQueue, System, SystemUID},
};

/// Logic of the demo.
#[derive(Debug)]
pub struct LogicSystem {
  uid: SystemUID,
  addr: Addr<LogicMsg>,
  msgs: MsgQueue<LogicMsg>,
  runtime_addr: Addr<RuntimeMsg>,
}

impl LogicSystem {
  pub fn new(runtime_addr: Addr<RuntimeMsg>, uid: SystemUID) -> Self {
    let (addr, msgs) = system_init(uid);
    Self {
      uid,
      addr,
      msgs,
      runtime_addr,
    }
  }
}

impl System for LogicSystem {
  type Addr = Addr<LogicMsg>;

  fn system_addr(&self) -> Self::Addr {
    self.addr.clone()
  }

  fn startup(self) {
    // we’ll live in our own thread thank you
    std::thread::spawn(move || loop {
      match self.msgs.recv() {
        Some(LogicMsg::Kill) => {
          self
            .runtime_addr
            .send_msg(RuntimeMsg::SystemExit(self.uid))
            .unwrap();
          break;
        }

        _ => (),
      }
    });
  }
}

#[derive(Debug)]
pub enum LogicMsg {
  Kill,
}

impl From<Kill> for LogicMsg {
  fn from(_: Kill) -> Self {
    Self::Kill
  }
}
