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
  let input = Pump::new().attach(&mut executor, tx);
  let received = executor.spawn(async move {
    input.send(1).await.unwrap();
    rx.receive().await.unwrap()
  });
  executor.run();
  assert_eq!(received.output(), Some(1));
}
