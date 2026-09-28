//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use netspin_core::Channel;
use netspin_core::Executor;
use netspin_core::PollSet;
use netspin_core::PollTarget;
use netspin_core::Pollable;
use netspin_core::Rx;

struct DelegatedPollable {
  rx: Rx<i32>,
}

impl Pollable for DelegatedPollable {
  fn poll_target(&self) -> PollTarget<'_> {
    self.rx.poll_target()
  }
}

// A downstream Pollable implementation delegates readiness.
#[test]
fn test() {
  let (tx, rx) = Channel::new().pair();
  let poll_set = PollSet::new();
  let object = poll_set.insert(DelegatedPollable { rx });
  let key = object.key();
  let mut executor = Executor::new();
  executor.spawn(async move {
    tx.send(7).await.unwrap();
  });
  let receiver = executor.spawn(async move {
    let ready = poll_set.poll().await;
    (ready, object.rx.expect_receive().unwrap())
  });
  executor.run();
  assert_eq!(receiver.output(), Some((key, 7)));
}
