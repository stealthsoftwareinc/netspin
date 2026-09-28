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

// A configured readiness threshold waits for that much free space.
#[test]
fn test() {
  let (tx, rx) = Channel::new().capacity(Some(4)).pair();
  tx.set_readiness_threshold(3);
  assert_eq!(tx.readiness_threshold(), 3);
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    for _ in 0..3 {
      Task::sleep(Duration::from_nanos(10)).await;
      rx.receive().await.unwrap();
    }
  });
  let sender = executor.spawn(async move {
    for value in 0..4 {
      tx.expect_send(value).unwrap();
    }
    assert_eq!(poll(&[&tx]).await, 0);
    Task::now()
  });
  executor.run();

  assert_eq!(receiver.output(), Some(()));
  assert_eq!(sender.output(), Some(Moment::from_nanos(30)));
}
