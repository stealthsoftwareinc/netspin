//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::convert::Infallible;
use core::error::Error;

use crate::Channel;
use crate::ChannelErrorInfo;
use crate::Moment;
use crate::PollTarget;
use crate::Pollable;
use crate::Rx;
use crate::Splice;
use crate::Split;
use crate::Tx;
use crate::channel_error::UnspecifiedChannelAbort;
use crate::poll_target_ops::sealed::PollTargetOps;

/// One endpoint of a bidirectional channel.
#[must_use]
pub struct Duplex<T, R = T, E: Error + 'static = Infallible> {
  /// Sends messages to the peer endpoint.
  pub tx: Tx<T, E>,

  /// Receives messages from the peer endpoint.
  pub rx: Rx<R, E>,
}

impl<T, R, E: Error + 'static> Duplex<T, R, E> {
  /// Creates a connected pair of duplex endpoints.
  pub fn pair() -> (Self, Duplex<R, T, E>) {
    let (a_tx, b_rx) = Channel::new().fallible().pair();
    let (b_tx, a_rx) = Channel::new().fallible().pair();
    let a = Duplex { tx: a_tx, rx: a_rx };
    let b = Duplex { tx: b_tx, rx: b_rx };
    (a, b)
  }

  /// Reports an untyped terminal failure in both directions.
  ///
  /// Messages already sent by this endpoint are received by the peer
  /// before it observes the abort.
  /// Messages awaiting receipt by this endpoint are discarded.
  /// Repeated calls have no effect.
  pub fn abort(&self) {
    self.abort_with(UnspecifiedChannelAbort);
  }

  /// Reports an untyped terminal failure caused by `error` in both
  /// directions.
  ///
  /// Messages already sent by this endpoint are received by the peer
  /// before it observes the abort.
  /// Messages awaiting receipt by this endpoint are discarded.
  /// Repeated calls have no effect.
  pub fn abort_with(&self, error: impl Into<ChannelErrorInfo>) {
    let info = error.into();
    self.tx.abort_with(info.clone());
    self.rx.abort_with(info);
  }

  /// Reports a terminal failure in both directions.
  ///
  /// Messages already sent by this endpoint are received by the peer
  /// before it observes the failure. Messages awaiting receipt by this
  /// endpoint are discarded. Repeated calls have no effect.
  pub fn fail(&self, error: impl Into<ChannelErrorInfo<E>>) {
    let info = error.into();
    self.tx.fail(info.clone());
    self.rx.fail(info);
  }
}

/// Waits until a message is ready to receive.
impl<T, R, E: Error + 'static> Pollable for Duplex<T, R, E> {
  fn poll_target(&self) -> PollTarget<'_> {
    PollTarget::new(self)
  }
}

impl<T, R, E: Error + 'static> PollTargetOps for Duplex<T, R, E> {
  fn declare_poller(&self) {
    self.rx.declare_poller();
  }

  fn ready_at(&self) -> Option<Moment> {
    self.rx.ready_at()
  }

  fn set_poll_task(&self) {
    self.rx.set_poll_task();
  }

  fn clear_poll_task(&self) {
    self.rx.clear_poll_task();
  }
}

impl<T, R, E: Error + 'static> Splice<Duplex<R, T, E>>
  for Duplex<T, R, E>
{
  fn splice(self, target: Duplex<R, T, E>) {
    self.tx.splice(target.rx);
    self.rx.splice(target.tx);
  }
}

impl<T, R, E: Error + 'static> Split for Duplex<T, R, E> {
  type Tx = Tx<T, E>;
  type Rx = Rx<R, E>;

  fn split(self) -> (Self::Tx, Self::Rx) {
    (self.tx, self.rx)
  }
}

#[cfg(test)]
mod tests;
