//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Attach;
use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Executor;
use crate::loss::FixedLoss;
use crate::loss::LossBlock;
use crate::test_error::TestError;

// An upstream abort is propagated after preceding messages are dropped.
#[test]
fn test() {
  let (input, target) = Channel::<u8, TestError>::default().pair();
  let mut executor = Executor::new();
  let output = LossBlock::new()
    .model(FixedLoss::new(1.0))
    .attach(&mut executor, target);
  let abort: ChannelErrorInfo = TestError("aborted").into();
  let reported_abort = abort.clone();
  executor.spawn(async move {
    input.send(7).await.unwrap();
    input.abort_with(reported_abort);
  });
  let receiver = executor.spawn(async move { output.receive().await });
  executor.run();
  assert_eq!(
    receiver.output(),
    Some(Err(ChannelError::Aborted(abort))),
  );
}
