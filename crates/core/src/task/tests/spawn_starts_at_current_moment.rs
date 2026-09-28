//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Executor;
use crate::Task;

// A task spawned after time has advanced starts at the spawning task's
// current moment, not at the beginning of the simulation.
#[test]
fn test() {
  let mut executor = Executor::new();
  let parent = executor.spawn(async {
    Task::sleep(Duration::from_secs(5)).await;
    Task::spawn(async { Task::now() })
  });
  executor.run();

  let child = parent.output().unwrap();
  assert_eq!(
    child.output().unwrap().to_duration(),
    Duration::from_secs(5),
  );
}
