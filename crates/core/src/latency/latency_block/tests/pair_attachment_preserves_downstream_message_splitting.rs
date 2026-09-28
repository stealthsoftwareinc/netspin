//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::latency::LatencyBlock;
use crate::poll;

// Pair attachment preserves a downstream channel's custom message
// splitting behavior.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (input, block_input) = Channel::new().pair();
  let (block_output, output) = Channel::new()
    .capacity(Some(4))
    .message_size(Vec::len)
    .message_split(|message: &mut Vec<_>, limit| {
      (limit < message.len()).then(|| message.split_off(limit))
    })
    .pair();
  LatencyBlock::new()
    .attach(&mut executor, (block_input, block_output));
  executor.spawn(async move {
    input.send(vec![1, 2, 3, 4]).await.unwrap();
  });
  let received = executor.spawn(async move {
    assert_eq!(poll(&[&output]).await, 0);
    let first = output.try_receive_up_to(2).unwrap().unwrap();
    let second = output.try_receive_up_to(2).unwrap().unwrap();
    (first, second)
  });
  executor.run();
  assert_eq!(received.output(), Some((vec![1, 2], vec![3, 4])));
}
