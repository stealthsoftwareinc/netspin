//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::FanOutBlock;
use crate::test_error::TestError;

// An upstream abort follows preceding messages to every output.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanOutBlock { app_tx, net_rxs } =
    FanOutBlock::<u8, TestError>::new(&mut executor, 2);
  let abort: ChannelErrorInfo = TestError("aborted").into();
  let reported_abort = abort.clone();
  executor.spawn(async move {
    app_tx.send(7).await.unwrap();
    app_tx.abort_with(reported_abort);
  });
  let receivers = executor.spawn(async move {
    let mut results = Vec::new();
    for net_rx in net_rxs {
      results.push((net_rx.receive().await, net_rx.receive().await));
    }
    results
  });
  executor.run();
  assert_eq!(
    receivers.output(),
    Some(vec![
      (Ok(7), Err(ChannelError::Aborted(abort.clone()))),
      (Ok(7), Err(ChannelError::Aborted(abort))),
    ]),
  );
}
