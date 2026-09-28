//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Rx::try_receive_up_to() returns a bounded prefix and retains the
// suffix of the front message.
#[test]
fn test() {
  let (tx, rx) = Channel::new()
    .capacity(Some(8))
    .message_size(Vec::len)
    .message_split(|bytes: &mut Vec<u8>, size| {
      Some(bytes.split_off(size))
    })
    .pair();
  let mut executor = Executor::new();
  let result = executor.spawn(async move {
    tx.send(vec![1, 2, 3, 4, 5]).await.unwrap();
    (rx.try_receive_up_to(3).unwrap(), rx.try_receive().unwrap())
  });
  executor.run();

  assert_eq!(
    result.output(),
    Some((Some(vec![1, 2, 3]), Some(vec![4, 5]))),
  );
}
