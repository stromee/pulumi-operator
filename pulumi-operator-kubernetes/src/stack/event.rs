use crate::stack::crd::PulumiStack;

pub enum PulumiStackChange {
  Added {
    added_stack: PulumiStack,
  },
  Removed {
    removed_stack: PulumiStack,
  },
  Updated {
    old_stack: PulumiStack,
    new_stack: PulumiStack,
  },
}
