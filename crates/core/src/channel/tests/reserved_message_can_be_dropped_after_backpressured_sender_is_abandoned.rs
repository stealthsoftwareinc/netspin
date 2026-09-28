//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use std::rc::Rc;

use crate::Channel;
use crate::Executor;

// Dropping a channel reservation after its backpressured sender has
// been abandoned does not require a running task.
#[test]
fn test() {
  let reservation_out = Rc::new(RefCell::new(None));
  let mut executor = Executor::new();
  executor.spawn({
    let reservation_out = reservation_out.clone();
    async move {
      let (tx, rx) = Channel::new().pair();
      tx.send(1).await.unwrap();
      let (_, reservation) = rx.expect_receive_reserved().unwrap();
      *reservation_out.borrow_mut() = Some(reservation);
      tx.send(2).await.unwrap();
    }
  });
  executor.run();
  drop(reservation_out.borrow_mut().take());
}
