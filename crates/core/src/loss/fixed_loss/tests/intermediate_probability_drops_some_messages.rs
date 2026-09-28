//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::Executor;
use crate::loss::FixedLoss;
use crate::loss::LossModel;

// A fixed loss model with an intermediate probability drops some
// messages and retains others.
#[test]
fn test() {
  let mut executor = Executor::new().rng(StdRng::seed_from_u64(0));
  let dropped = executor.spawn(async move {
    let mut model = FixedLoss::new(0.5);
    (0..100).filter(|_| model.should_drop()).count()
  });
  executor.run();
  let dropped = dropped.output().unwrap();
  assert!(dropped > 0);
  assert!(dropped < 100);
}
