//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use std::sync::Arc;
use std::task::Wake;

use super::super::TaskWaker;
use super::super::WOKEN;

// A wake call made outside a running task is ignored.
#[test]
fn test() {
  Arc::new(TaskWaker {
    key: crate::task_key::TaskKey {
      id: 0,
      slot: core::num::NonZeroUsize::new(1).unwrap(),
    },
  })
  .wake();
  WOKEN.with_borrow(|woken| assert!(woken.is_empty()));
}
