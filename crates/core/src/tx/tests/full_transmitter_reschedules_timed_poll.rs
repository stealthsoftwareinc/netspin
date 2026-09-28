//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll_with_timeout;

// Freeing channel space reschedules a timed transmitter poll.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    rx.receive().await.unwrap()
  });
  let sender = executor.spawn(async move {
    tx.expect_send(1).unwrap();
    let ready =
      poll_with_timeout(&[&tx], Duration::from_nanos(20)).await;
    (ready, Task::now())
  });
  executor.run();

  assert_eq!(receiver.output(), Some(1));
  assert_eq!(sender.output(), Some((Some(0), Moment::from_nanos(10))));
}
