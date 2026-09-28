//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::PollSet;
use crate::SendOptions;
use crate::Task;

// Dropping an entry after its unparked poll is abandoned does not
// require a running task.
#[test]
fn test() {
  let mut executor = Executor::new().horizon(Duration::from_nanos(5));
  executor.spawn(async {
    let poll_set = PollSet::new();
    let (tx, rx) =
      Channel::<i64>::new().capacity(None).local(false).pair();
    let options = SendOptions::new().latency(Duration::from_nanos(10));
    tx.send_with_options(1, options).await.unwrap();
    let _entry = poll_set.insert(rx);
    let _ = poll_set.poll().await;
  });
  executor.spawn(async {
    Task::sleep(Duration::from_nanos(5)).await;
  });
  executor.run();
}
