//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::Channel;
use crate::ChannelError;
use crate::Executor;
use crate::test_error::TestError;

// A reserved receive returns a ready channel failure.
#[test]
fn test() {
  let mut executor = Executor::new();
  executor.spawn(async {
    let (_tx, rx) = Channel::<()>::new().fallible::<TestError>().pair();
    rx.fail(TestError("failure"));
    assert!(matches!(
      rx.expect_receive_reserved(),
      Err(ChannelError::Failed(info))
        if *info == TestError("failure"),
    ));
  });
  executor.run();
}
