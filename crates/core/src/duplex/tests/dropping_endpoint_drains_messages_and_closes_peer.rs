//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::Duplex;
use crate::Executor;
use crate::SendError;
use crate::test_error::TestError;

// Dropping an endpoint lets its peer drain messages and closes it.
#[test]
fn test() {
  let (a, b) = Duplex::<u8, u8, TestError>::pair();
  let mut executor = Executor::new();
  executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    drop(a);
  });
  let peer = executor.spawn(async move {
    (
      b.rx.receive().await,
      b.rx.receive().await,
      b.tx.send(2).await,
    )
  });
  executor.run();

  assert_eq!(
    peer.output(),
    Some((
      Ok(1),
      Err(ChannelError::Closed),
      Err(SendError {
        cause: ChannelError::Closed,
        sent: 0,
        unsent: 2,
      }),
    )),
  );
}
