//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;
use crate::PollSet;

// A transmitter stored in a PollSet remains directly usable through
// its entry handle.
#[test]
fn test() {
  let (tx, _rx) = Channel::new().pair();
  let mut executor = Executor::new();
  let sender = executor.spawn(async move {
    let poll_set = PollSet::new();
    let tx = poll_set.insert(tx);
    let key = tx.key();
    assert_eq!(poll_set.poll().await, key);
    tx.expect_send(1).unwrap();
    tx.free_space()
  });
  executor.run();

  assert_eq!(sender.output(), Some(0));
}
