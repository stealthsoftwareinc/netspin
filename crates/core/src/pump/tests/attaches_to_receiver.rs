//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::Executor;
use crate::Pump;

#[test]
fn test() {
  let mut executor = Executor::new();
  let (tx, rx) = Channel::new().pair();
  let output = Pump::new().attach(&mut executor, rx);
  let received = executor.spawn(async move {
    tx.send(1).await.unwrap();
    output.receive().await.unwrap()
  });
  executor.run();
  assert_eq!(received.output(), Some(1));
}
