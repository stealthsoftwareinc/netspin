//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// A message_split function that declines to split a message when the
// channel is empty panics, as this means the message can never fit.
#[test]
#[should_panic(expected = "must not be impossibly large")]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::<Vec<u8>>::new()
    .capacity(Some(4))
    .local(false)
    .message_size(|m: &Vec<u8>| m.len())
    .message_split(|_, _| None)
    .pair();
  executor.spawn(async move {
    let _ = tx.try_send(vec![0; 5]).unwrap();
  });
  executor.run();
}
