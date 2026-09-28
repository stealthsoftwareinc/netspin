//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;
use crate::Executor;
use crate::SendOptions;

// Sending a message into a local channel with nonzero latency panics.
#[test]
#[should_panic(expected = "must use zero latency")]
fn test() {
  let mut executor = Executor::new();
  let (tx, _rx) = Channel::new().pair();
  executor.spawn(async move {
    let options = SendOptions::new().latency(Duration::from_secs(1));
    tx.send_with_options(7, options).await.unwrap();
  });
  executor.run();
}
