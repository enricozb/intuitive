use crate::{
  components::{AnyComponent, Component},
  element::Any as AnyElement,
};

#[derive(Default)]
pub struct Embed {
  pub component: Option<AnyComponent>,
  pub element: Option<AnyElement>,
}

impl Component for Embed {
  fn render(&self) -> AnyElement {
    match (&self.component, &self.element) {
      (Some(_), Some(_)) => panic!("Embed can only take one of its arguments"),

      (Some(component), _) => component.render(),
      (_, Some(element)) => Clone::clone(element),

      _ => AnyElement::default(),
    }
  }
}
