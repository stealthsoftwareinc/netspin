//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// Sending a message into a local channel with zero latency works.
#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().pair();
  executor.spawn(async move {
    tx.send(7).await.unwrap();
  });
  let receiver =
    executor.spawn(async move { rx.receive().await.unwrap() });
  executor.run();
  assert_eq!(receiver.output().unwrap(), 7);
}
