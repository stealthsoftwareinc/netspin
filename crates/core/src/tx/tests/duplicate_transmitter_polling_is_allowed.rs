//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;
use crate::poll;
use crate::poll_with_timeout;

// Polling the same transmitter more than once supports both timed and
// parked polls and returns its first index.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    let first = rx.receive().await.unwrap();
    Task::sleep(Duration::from_nanos(10)).await;
    let second = rx.receive().await.unwrap();
    [first, second]
  });
  let sender = executor.spawn(async move {
    tx.expect_send(1).unwrap();
    let timed =
      poll_with_timeout(&[&tx, &tx], Duration::from_nanos(20)).await;
    tx.expect_send(2).unwrap();
    let parked = poll(&[&tx, &tx]).await;
    (timed, parked, Task::now())
  });
  executor.run();

  assert_eq!(receiver.output(), Some([1, 2]));
  assert_eq!(
    sender.output(),
    Some((Some(0), 0, Moment::from_nanos(20))),
  );
}
