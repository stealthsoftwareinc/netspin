//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Duplex;
use crate::Executor;
use crate::PollSet;

// A Duplex becomes ready when it has a message to receive.
#[test]
fn test() {
  let mut executor = Executor::new();
  let poll_set = PollSet::new();
  let (duplex, peer) = Duplex::<(), i32>::pair();
  let duplex = poll_set.insert(duplex);
  let key = duplex.key();
  executor.spawn(async move {
    peer.tx.send(7).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, duplex.rx.expect_receive().unwrap())
  });
  executor.run();
  assert_eq!(receiver.output(), Some((key, 7)));
}
