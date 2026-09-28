//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::executor::SUSPEND;
use crate::executor::Suspend;

// Starting a task poll with a previously registered suspension panics.
#[test]
#[should_panic(
  expected = "Async rule violation: A task suspension was already \
              registered at the start of a poll."
)]
fn test() {
  SUSPEND.set(Some(Suspend::Park));
  let mut executor = Executor::new();
  executor.spawn(async {});
  executor.run();
}
