//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::loss::LossBlock;

// Attaching a default LossBlock to a receiver forwards every message.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (input, target) = Channel::new().pair();
  let output = LossBlock::new().attach(&mut executor, target);
  executor.spawn(async move {
    for i in 0..10u32 {
      input.send(i).await.unwrap();
    }
  });
  let received = executor.spawn(async move {
    let mut messages = Vec::new();
    for _ in 0..10 {
      messages.push(output.receive().await.unwrap());
    }
    messages
  });
  executor.run();
  assert_eq!(received.output(), Some((0..10).collect::<Vec<u32>>()));
}
