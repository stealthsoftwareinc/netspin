//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::FanOutBlock;

// A FanOutBlock requires at least one output.
#[test]
#[should_panic(
  expected = "A FanOutBlock must have at least one output."
)]
fn test() {
  let mut executor = Executor::new();
  let _ = FanOutBlock::<()>::new(&mut executor, 0);
}
