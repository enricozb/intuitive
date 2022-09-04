use std::sync::Arc;

use parking_lot::Mutex;

use crate::{components::AnyComponent, state::State};

static FUNCS: Mutex<Option<Funcs>> = Mutex::new(None);

#[derive(Clone)]
pub struct Funcs {
  show: Arc<dyn Fn(AnyComponent) + Send + Sync>,
  hide: Arc<dyn Fn() + Send + Sync>,
}

impl Funcs {
  pub fn new(modal: State<Option<AnyComponent>>) -> Self {
    let show_modal = modal.clone();
    let hide_modal = modal;

    Self {
      show: Arc::new(move |component| show_modal.set(Some(component))),
      hide: Arc::new(move || hide_modal.set(None)),
    }
  }

  pub fn show(&self, component: AnyComponent) {
    (self.show)(component)
  }

  pub fn hide(&self) {
    (self.hide)()
  }
}

pub fn use_modal_funcs() -> Funcs {
  FUNCS
    .lock()
    .clone()
    .expect("use modal called outside of a Modal or outside of render")
}

pub fn set_modal_funcs(funcs: Funcs) {
  *FUNCS.lock() = Some(funcs)
}
