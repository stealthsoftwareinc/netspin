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

// Releasing reserved channel space makes a full transmitter ready.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(5)).await;
    let (message, reservation) = rx.expect_receive_reserved().unwrap();
    Task::sleep(Duration::from_nanos(5)).await;
    drop(reservation);
    message
  });
  let sender = executor.spawn(async move {
    tx.expect_send(1).unwrap();
    assert_eq!(poll(&[&tx]).await, 0);
    Task::now()
  });
  executor.run();

  assert_eq!(receiver.output(), Some(1));
  assert_eq!(sender.output(), Some(Moment::from_nanos(10)));
}
