use crate::{
  buffer::region::Region,
  components::Component,
  element::{Any as AnyElement, Element},
  error::Result,
  utils::layout::{Flex, FlexDirection},
};

/// A component that renders stacks of its children.
#[derive(Clone)]
pub struct Stack<const N: usize> {
  pub direction: FlexDirection,
  pub flex: [Flex; N],
  pub children: [AnyElement; N],
}

impl<const N: usize> Default for Stack<N> {
  fn default() -> Self {
    Self {
      direction: FlexDirection::Row,
      flex: [(); N].map(|()| Flex::default()),
      children: [(); N].map(|()| AnyElement::default()),
    }
  }
}

impl<const N: usize> Component for Stack<N> {
  fn render(&self) -> AnyElement {
    AnyElement::new(self.clone())
  }
}

impl<const N: usize> Element for Stack<N> {
  fn draw<'a>(&self, region: &'a mut Region<'a>) -> Result<()> {
    let region_size = region.size();

    let total = match self.direction {
      FlexDirection::Row => region_size.width,
      FlexDirection::Column => region_size.height,
    };

    let total_rel: u16 = self.flex.iter().map(Flex::grow).sum();
    let total_fixed: u16 = self.flex.iter().map(Flex::fixed).sum();

    let mut total_offset = 0;
    for i in 0..N {
      let cur_offset = match self.flex[i] {
        Flex::Grow(rel) => (total - total_fixed) * rel / total_rel,
        Flex::Fixed(fixed) => fixed,
      };

      let (child_position, child_size) = match self.direction {
        FlexDirection::Row => ((total_offset, 0), (cur_offset, region_size.height)),
        FlexDirection::Column => ((0, total_offset), (region_size.width, cur_offset)),
      };

      self.children[i].draw(&mut region.narrow(child_position.into(), child_size.into())?)?;

      total_offset += cur_offset;
    }

    Ok(())
  }
}
