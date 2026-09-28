//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Executor;
use crate::Task;

// Task::block_on() runs a future to completion.
#[test]
fn test() {
  let mut executor = Executor::new();
  let output = executor.spawn(async {
    Task::block_on(async { tokio::spawn(async { 7 }).await.unwrap() })
  });
  executor.run();

  assert_eq!(output.output(), Some(7));
}
