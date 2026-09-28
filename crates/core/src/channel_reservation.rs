//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::convert::Infallible;
use core::error::Error;

use crate::channel_state::ChannelRef;

/// Retains space for a message removed from a channel.
#[must_use]
pub struct ChannelReservation<T, E: Error + 'static = Infallible> {
  channel: ChannelRef<T, E>,
  size: usize,
}

impl<T, E: Error + 'static> ChannelReservation<T, E> {
  pub(super) fn new(channel: ChannelRef<T, E>, size: usize) -> Self {
    Self { channel, size }
  }

  /// Releases some of the retained channel space.
  pub fn release(&mut self, size: usize) {
    assert!(
      size <= self.size,
      "A channel reservation cannot release more than it retains.",
    );
    if size == 0 {
      return;
    }
    let mut channel = self.channel.borrow_mut();
    assert!(
      size <= channel.queued_size,
      "A channel reservation must retain queued space.",
    );
    channel.queued_size -= size;
    self.size -= size;
    channel.interrupt_sender();
  }

  /// Returns the amount of channel space retained.
  #[must_use]
  pub fn size(&self) -> usize {
    self.size
  }
}

impl<T, E: Error + 'static> Drop for ChannelReservation<T, E> {
  fn drop(&mut self) {
    self.release(self.size);
  }
}
