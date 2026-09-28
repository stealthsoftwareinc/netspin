//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::time::Duration;
use std::rc::Rc;

use crate::Executor;

// An executor with a zero horizon does not start tasks scheduled at
// the initial moment.
#[test]
fn test() {
  let mut executor = Executor::new().horizon(Duration::ZERO);
  let started = Rc::new(Cell::new(false));
  let output = executor.spawn({
    let started = started.clone();
    async move {
      started.set(true);
    }
  });

  executor.run();

  assert!(!started.get());
  assert_eq!(output.output(), None);
}
