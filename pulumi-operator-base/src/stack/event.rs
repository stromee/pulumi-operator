use crate::stack::cached_stack::CachedPulumiStack;

pub enum PulumiStackChange {
  Added {
    added_stack: CachedPulumiStack,
  },
  Removed {
    removed_stack: CachedPulumiStack,
  },
  Updated {
    old_stack: CachedPulumiStack,
    new_stack: CachedPulumiStack,
  },
}
