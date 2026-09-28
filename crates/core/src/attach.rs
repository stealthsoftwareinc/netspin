//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Spawn;

/// A block that can be attached to an existing target.
pub trait Attach<Target> {
  /// The interface provided by the attached block.
  type Surface;

  /// Attaches this block to the given target, using the given spawner
  /// to spawn any necessary tasks.
  fn attach(
    self,
    spawner: &mut impl Spawn,
    target: Target,
  ) -> Self::Surface;
}
