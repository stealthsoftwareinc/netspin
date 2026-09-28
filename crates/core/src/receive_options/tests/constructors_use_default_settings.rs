//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::ReceiveOptions;

// ReceiveOptions constructors use no timeout.
#[test]
fn test() {
  for options in [ReceiveOptions::new(), ReceiveOptions::default()] {
    assert_eq!(options.timeout, None);
  }
}
