//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Duplex;
use crate::Executor;
use crate::SendError;
use crate::test_error::TestError;

// A duplex failure drains outgoing messages and fails both directions.
#[test]
fn test() {
  let (a, b) = Duplex::<u8, u8, TestError>::pair();
  let mut executor = Executor::new();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  let failing = executor.spawn(async move {
    a.tx.send(1).await.unwrap();
    a.fail(reported_failure);
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

  assert_eq!(
    failing.output(),
    Some(Err(ChannelError::Failed(failure.clone()))),
  );
  assert_eq!(
    peer.output(),
    Some((
      Ok(1),
      Err(ChannelError::Failed(failure.clone())),
      Err(SendError {
        cause: ChannelError::Failed(failure),
        sent: 0,
        unsent: 2,
      }),
    )),
  );
}
