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

// A full channel's transmitter becomes ready when a receive frees
// space.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(10)).await;
    let first = rx.receive().await.unwrap();
    let second = rx.receive().await.unwrap();
    [first, second]
  });
  let sender = executor.spawn(async move {
    tx.expect_send(1).unwrap();
    assert_eq!(poll(&[&tx]).await, 0);
    tx.expect_send(2).unwrap();
    Task::now()
  });
  executor.run();

  assert_eq!(receiver.output(), Some([1, 2]));
  assert_eq!(sender.output(), Some(Moment::from_nanos(10)));
}
