//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// A send into an almost-full channel with a message_split function
// splits the message to send as much as possible.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::<Vec<u8>>::new()
    .capacity(Some(4))
    .local(false)
    .message_size(|m: &Vec<u8>| m.len())
    .message_split(|m, size| Some(m.split_off(size)))
    .pair();
  let sender = executor.spawn(async move {
    tx.try_send(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]).unwrap()
  });
  let receiver =
    executor.spawn(async move { rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(sender.output().unwrap(), Some(vec![4, 5, 6, 7, 8, 9]));
  assert_eq!(receiver.output().unwrap(), [0, 1, 2, 3]);
}
