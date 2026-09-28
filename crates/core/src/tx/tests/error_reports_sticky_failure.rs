//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::test_error::TestError;

// Tx::error() reports the channel's sticky failure.
#[test]
fn test() {
  let (tx, rx) = Channel::<()>::new().fallible::<TestError>().pair();
  assert_eq!(tx.error(), None);
  let failure = ChannelErrorInfo::new(TestError("failed"));
  rx.fail(failure.clone());
  assert_eq!(tx.error(), Some(ChannelError::Failed(failure.clone())));
  assert_eq!(tx.error(), Some(ChannelError::Failed(failure)));
}
