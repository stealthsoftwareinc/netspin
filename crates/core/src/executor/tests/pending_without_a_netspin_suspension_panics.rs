//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::future::pending;

use crate::Executor;

// Returning Pending without a NetSpin suspension panics.
#[test]
#[should_panic(
  expected = "Async rule violation: A task returned pending without \
              registering a suspension."
)]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(pending::<()>());
  executor.run();
}
