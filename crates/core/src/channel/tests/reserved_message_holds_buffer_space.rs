//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::Executor;

// A reserved message continues occupying channel space until its
// reservation is dropped.
#[test]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(async {
    let (tx, rx) = Channel::new().capacity(Some(2)).pair();
    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();

    let (message, reservation) = rx.expect_receive_reserved().unwrap();
    assert_eq!(message, 1);
    assert_eq!(tx.try_send(3).unwrap(), Some(3));

    drop(reservation);
    assert_eq!(tx.try_send(3).unwrap(), None);
  });
  executor.run();
}
