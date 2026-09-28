//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::Moment;
use crate::Task;

// Active work advances only the calling task's clock while other tasks
// continue to run during the interval.
#[test]
fn test() {
  let mut executor = Executor::new();
  let worker = executor.spawn(async {
    let before = Task::now();
    Task::work(Duration::from_nanos(5)).await;
    (before, Task::now())
  });
  let observer = executor.spawn(async {
    Task::sleep(Duration::from_nanos(2)).await;
    Task::now()
  });
  executor.run();

  assert_eq!(
    worker.output(),
    Some((Moment::ZERO, Moment::from_nanos(5)))
  );
  assert_eq!(observer.output(), Some(Moment::from_nanos(2)));
}
