//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

/// A value that can be spliced with a `Target`.
pub trait Splice<Target> {
  /// Splices this value with `target`.
  fn splice(self, target: Target);
}
