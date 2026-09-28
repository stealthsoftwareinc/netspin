//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::FanInBlock;
use crate::test_error::TestError;

// An input failure follows messages from every preceding input.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanInBlock {
    mut app_txs,
    net_rx,
  } = FanInBlock::<u8, TestError>::new(&mut executor, 2);
  let second = app_txs.pop().unwrap();
  let first = app_txs.pop().unwrap();
  let failure = ChannelErrorInfo::new(TestError("failed"));
  let reported_failure = failure.clone();
  executor.spawn(async move {
    first.send(0).await.unwrap();
  });
  executor.spawn(async move {
    second.send(1).await.unwrap();
    second.fail(reported_failure);
  });
  let receiver = executor.spawn(async move {
    (
      net_rx.receive().await,
      net_rx.receive().await,
      net_rx.receive().await,
    )
  });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some((Ok(0), Ok(1), Err(ChannelError::Failed(failure)),)),
  );
}
