//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::time::Duration;
use std::rc::Rc;

use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::Executor;
use crate::Task;

// Task::rng() draws from the executor's random number generator, and the
// same handle keeps working when it is held across an await point.
#[test]
fn test() {
  let mut executor = Executor::new().rng(StdRng::seed_from_u64(17));
  let drawn = Rc::new(RefCell::new(Vec::new()));
  {
    let drawn = drawn.clone();
    executor.spawn(async move {
      let mut rng = Task::rng();
      for _ in 0..100 {
        drawn.borrow_mut().push(rng.random_range(0..u64::MAX));
        Task::sleep(Duration::ZERO).await;
      }
    });
  }
  executor.run();

  let mut expected = StdRng::seed_from_u64(17);
  let expected: Vec<u64> = (0..100)
    .map(|_| expected.random_range(0..u64::MAX))
    .collect();
  assert_eq!(*drawn.borrow(), expected);
}
