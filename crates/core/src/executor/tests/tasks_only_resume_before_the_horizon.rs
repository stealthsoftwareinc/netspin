//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::Task;

// Tasks scheduled before an executor's horizon resume, while tasks
// scheduled at or beyond it do not.
#[test]
fn test() {
  let mut executor = Executor::new().horizon(Duration::from_nanos(10));
  let before = executor.spawn(async {
    Task::sleep(Duration::from_nanos(9)).await;
    Task::now().to_duration()
  });
  let at = executor.spawn(async {
    Task::sleep(Duration::from_nanos(10)).await;
  });
  let beyond = executor.spawn(async {
    Task::sleep(Duration::from_nanos(11)).await;
  });

  executor.run();

  assert_eq!(before.output(), Some(Duration::from_nanos(9)));
  assert_eq!(at.output(), None);
  assert_eq!(beyond.output(), None);
}
