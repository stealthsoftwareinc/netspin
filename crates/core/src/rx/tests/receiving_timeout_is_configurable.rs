//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::time::Duration;

use crate::Channel;

// Receiving timeouts default to none and normalize zero to none.
#[test]
fn test() {
  let (_tx, rx) = Channel::<()>::new().pair();

  assert_eq!(rx.receive_timeout(), None);
  rx.set_receive_timeout(Some(Duration::from_nanos(7)));
  assert_eq!(rx.receive_timeout(), Some(Duration::from_nanos(7)));
  rx.set_receive_timeout(Some(Duration::ZERO));
  assert_eq!(rx.receive_timeout(), None);
  rx.set_receive_timeout(Some(Duration::from_nanos(8)));
  rx.set_receive_timeout(None);
  assert_eq!(rx.receive_timeout(), None);
}
