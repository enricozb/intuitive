use crate::{
  components::{Any as AnyComponent, Component},
  element::Any as AnyElement,
};

#[derive(Default)]
pub struct Embed {
  // TODO(enricozb): make this a single enum field with variants for each,
  //                 and implement Into for the enum from both Any's.
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
