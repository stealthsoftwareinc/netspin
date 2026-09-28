//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::SendError;
use crate::latency::LatencyBlock;
use crate::test_error::TestError;

// A downstream abort interrupts and aborts the upstream sender.
#[test]
fn test() {
  let (input, target) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  let output = LatencyBlock::new().attach(&mut executor, target);
  let abort: ChannelErrorInfo = TestError("aborted").into();
  output.abort_with(abort.clone());
  let sender = executor.spawn(async move {
    let first = input.send(0).await;
    let second = input.send(1).await;
    (first, second)
  });
  executor.run();
  assert_eq!(
    sender.output(),
    Some((
      Ok(()),
      Err(SendError {
        cause: ChannelError::Aborted(abort),
        sent: 0,
        unsent: 1,
      }),
    )),
  );
}
