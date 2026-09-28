//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// A bounded channel greedily accepts every message that fits.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<usize>::new()
    .capacity(Some(10))
    .local(false)
    .message_size(|&m| m)
    .pair();
  let sender = executor.spawn(async move {
    [
      tx.try_send(7).unwrap(),
      tx.try_send(4).unwrap(),
      tx.try_send(3).unwrap(),
      tx.try_send(1).unwrap(),
      tx.try_send(0).unwrap(),
    ]
  });
  let receiver = executor.spawn(async move {
    let mut log = Vec::new();
    for _ in 0..3 {
      log.push(rx.receive().await.unwrap());
    }
    log
  });
  executor.run();
  assert_eq!(
    sender.output().unwrap(),
    [None, Some(4), None, Some(1), None]
  );
  assert_eq!(receiver.output().unwrap(), [7, 3, 0]);
}
