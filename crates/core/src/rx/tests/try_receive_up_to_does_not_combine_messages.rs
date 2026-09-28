//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Rx::try_receive_up_to() does not combine separate ready messages
// when the first message is smaller than the maximum.
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
    tx.send(vec![1, 2]).await.unwrap();
    tx.send(vec![3, 4]).await.unwrap();
    (rx.try_receive_up_to(4).unwrap(), rx.try_receive().unwrap())
  });
  executor.run();

  assert_eq!(
    result.output(),
    Some((Some(vec![1, 2]), Some(vec![3, 4]))),
  );
}
