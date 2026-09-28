//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::loss::FixedLoss;
use crate::loss::LossModel;

// A fixed loss model with probability 0.0 drops no messages.
#[test]
fn test() {
  let mut executor = Executor::new();
  let dropped = executor.spawn(async move {
    let mut model = FixedLoss::new(0.0);
    (0..10).filter(|_| model.should_drop()).count()
  });
  executor.run();
  assert_eq!(dropped.output(), Some(0));
}
