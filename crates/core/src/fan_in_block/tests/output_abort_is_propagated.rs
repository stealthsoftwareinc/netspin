//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::FanInBlock;
use crate::SendError;
use crate::test_error::TestError;

// An output abort interrupts and aborts every input sender.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanInBlock {
    mut app_txs,
    net_rx,
  } = FanInBlock::<u8, TestError>::new(&mut executor, 2);
  let abort: ChannelErrorInfo = TestError("aborted").into();
  net_rx.abort_with(abort.clone());
  let passive = app_txs.pop().unwrap();
  let active = app_txs.pop().unwrap();
  let sender = executor.spawn(async move {
    let first = active.send(0).await;
    let second = active.send(1).await;
    (first, second)
  });
  executor.run();
  assert_eq!(
    sender.output(),
    Some((
      Ok(()),
      Err(SendError {
        cause: ChannelError::Aborted(abort.clone()),
        sent: 0,
        unsent: 1,
      }),
    )),
  );
  assert_eq!(passive.error(), Some(ChannelError::Aborted(abort)));
}
