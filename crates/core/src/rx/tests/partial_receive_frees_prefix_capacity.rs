//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::Moment;
use crate::Task;

// Partially receiving a message frees the capacity occupied by its
// returned prefix while retaining the suffix's capacity.
#[test]
fn test() {
  let (tx, rx) = Channel::new()
    .capacity(Some(5))
    .message_size(Vec::len)
    .message_split(|bytes: &mut Vec<u8>, size| {
      Some(bytes.split_off(size))
    })
    .pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    tx.send(vec![1, 2, 3, 4, 5]).await.unwrap();
    tx.send(vec![6, 7]).await.unwrap();
    Task::now()
  });
  let receiver = executor.spawn(async move {
    Task::sleep(Duration::from_nanos(1)).await;
    let prefix = rx.try_receive_up_to(2).unwrap();
    Task::sleep(Duration::from_nanos(1)).await;
    let suffix = rx.try_receive().unwrap();
    let next = rx.try_receive().unwrap();
    (prefix, suffix, next)
  });
  executor.run();

  assert_eq!(sender.output(), Some(Moment::from_nanos(1)));
  assert_eq!(
    receiver.output(),
    Some((Some(vec![1, 2]), Some(vec![3, 4, 5]), Some(vec![6, 7]),)),
  );
}
