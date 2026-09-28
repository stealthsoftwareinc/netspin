//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::Duplex;
use crate::Executor;
use crate::SendError;
use crate::test_error::TestError;

// A duplex abort drains outgoing messages and aborts both directions.
#[test]
fn test() {
  let (a, b) = Duplex::<u8, u8, TestError>::pair();
  let mut executor = Executor::new();
  let aborting = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    a.abort();
    a.rx.receive().await
  });
  let peer = executor.spawn(async move {
    (
      b.rx.receive().await,
      b.rx.receive().await,
      b.tx.send(2).await,
    )
  });
  executor.run();

  let abort = match aborting.output().unwrap().unwrap_err() {
    ChannelError::Aborted(info) => info,
    error => panic!("unexpected error: {error}"),
  };
  assert_eq!(
    peer.output(),
    Some((
      Ok(1),
      Err(ChannelError::Aborted(abort.clone())),
      Err(SendError {
        cause: ChannelError::Aborted(abort),
        sent: 0,
        unsent: 2,
      }),
    )),
  );
}
