//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::FanOutBlock;
use crate::SendError;
use crate::test_error::TestError;

// A downstream abort interrupts the sender and aborts every output.
#[test]
fn test() {
  let mut executor = Executor::new();
  let FanOutBlock {
    app_tx,
    mut net_rxs,
  } = FanOutBlock::<u8, TestError>::new(&mut executor, 2);
  let other = net_rxs.pop().unwrap();
  let aborted = net_rxs.pop().unwrap();
  let abort: ChannelErrorInfo = TestError("aborted").into();
  aborted.abort_with(abort.clone());
  let sender = executor.spawn(async move {
    let first = app_tx.send(0).await;
    let second = app_tx.send(1).await;
    (first, second)
  });
  let receiver = executor.spawn(async move {
    (aborted.receive().await, other.receive().await)
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
  assert_eq!(
    receiver.output(),
    Some((
      Err(ChannelError::Aborted(abort.clone())),
      Err(ChannelError::Aborted(abort)),
    )),
  );
}
