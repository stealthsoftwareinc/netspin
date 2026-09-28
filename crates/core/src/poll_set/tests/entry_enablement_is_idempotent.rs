//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use crate::PollSet;
use crate::Signal;

// PollSet entry enablement is idempotent and preserves the entry's key
// and object.
#[test]
fn test() {
  let poll_set = PollSet::new();
  let (trigger, signal) = Signal::pair();
  let signal = poll_set.insert(signal);
  let key = signal.key();

  assert!(signal.is_enabled());
  signal.enable();
  assert!(signal.is_enabled());

  signal.disable();
  signal.disable();
  assert!(!signal.is_enabled());
  assert_eq!(signal.key(), key);

  trigger.trigger();
  assert!(signal.is_triggered());

  signal.enable();
  signal.enable();
  assert!(signal.is_enabled());
  assert_eq!(signal.key(), key);
}
