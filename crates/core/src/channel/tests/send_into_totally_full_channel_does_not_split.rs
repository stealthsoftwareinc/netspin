//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// A send into a totally full channel with a message_split function does
// not try to split the message.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<Vec<u8>>::new()
    .capacity(Some(4))
    .local(false)
    .message_size(|m: &Vec<u8>| m.len())
    .message_split(|_, _| panic!("Unexpected message_split call"))
    .pair();
  let sender = executor.spawn(async move {
    [
      tx.try_send(vec![0, 1, 2, 3]).unwrap(),
      tx.try_send(vec![4, 5, 6]).unwrap(),
    ]
  });
  let receiver =
    executor.spawn(async move { rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(sender.output().unwrap(), [None, Some(vec![4, 5, 6])]);
  assert_eq!(receiver.output().unwrap(), [0, 1, 2, 3]);
}
