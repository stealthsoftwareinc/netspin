//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::SendOptions;

// SendOptions constructors use zero latency and no timeout.
#[test]
fn test() {
  for options in [SendOptions::new(), SendOptions::default()] {
    assert_eq!(options.latency, Duration::ZERO);
    assert_eq!(options.timeout, None);
  }
}
