//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::convert::Infallible;
use core::error::Error;
use core::time::Duration;
use std::rc::Rc;

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::Moment;
use crate::PollTarget;
use crate::Pollable;
use crate::Rx;
use crate::SendError;
use crate::SendOptions;
use crate::Splice;
use crate::channel_error::UnspecifiedChannelAbort;
use crate::channel_state::ChannelRef;
use crate::channel_state::ChannelState;
use crate::channel_state::SendState;
use crate::channel_state::TerminatedBy;
use crate::channel_state::TerminationReason;
use crate::channel_state::TxChannelRef;
use crate::executor::reschedule;
use crate::poll_target_ops::sealed::PollTargetOps;
use crate::task::Task;
use crate::task::TaskId;

/// The sending endpoint of a [`Channel`].
///
/// A transmitter can be polled for write readiness.
/// It is ready whenever the channel has at least one unit of free
/// space or has terminated.
/// Readiness is level-triggered and does not reserve channel space or
/// guarantee that a particular message will fit.
///
/// All send operations and readiness polls on an endpoint must be
/// performed by the same task.
///
/// [`Channel`]: crate::Channel
#[must_use]
pub struct Tx<T, E: Error + 'static = Infallible> {
  channel: Option<TxChannelRef<T, E>>,
  task_id: Cell<Option<TaskId>>,

  //--------------------------------------------------------------------
  // Options
  //--------------------------------------------------------------------
  send_timeout: Cell<Option<Duration>>,
}

impl<T, E: Error + 'static> Tx<T, E> {
  pub(super) fn new(channel: TxChannelRef<T, E>) -> Self {
    Self {
      channel: Some(channel),
      task_id: Cell::new(None),
      send_timeout: Cell::new(None),
    }
  }

  fn channel(&self) -> ChannelRef<T, E> {
    self.channel_ref().borrow().clone()
  }

  fn channel_ref(&self) -> &TxChannelRef<T, E> {
    self
      .channel
      .as_ref()
      .expect("A spliced transmitter must not be used")
  }

  fn with_channel<R>(
    &self,
    f: impl FnOnce(&ChannelState<T, E>) -> R,
  ) -> R {
    let channel = self.channel_ref().borrow();
    let channel = channel.borrow();
    f(&channel)
  }

  fn with_channel_mut<R>(
    &self,
    f: impl FnOnce(&mut ChannelState<T, E>) -> R,
  ) -> R {
    let channel = self.channel_ref().borrow();
    let mut channel = channel.borrow_mut();
    f(&mut channel)
  }

  pub(crate) fn is_spliceable_downstream(&self) -> bool {
    self.with_channel(|channel| {
      channel.queue.is_empty()
        && channel.queued_size == 0
        && matches!(&channel.send_state, SendState::Idle)
        && channel.rx_task.is_none()
        && channel.termination.is_none()
        && channel.local
        && channel.message_size.is_none()
        && channel.message_split.is_none()
    })
  }

  fn declare_sender(&self) {
    if cfg!(debug_assertions) {
      let task_id = Task::id();
      if let Some(expected) = self.task_id.get() {
        debug_assert!(
          task_id == expected,
          "A channel must not have multiple senders",
        );
        return;
      }
      self.task_id.set(Some(task_id));
    }
  }

  fn clear_send_state(&self) {
    self.with_channel_mut(|channel| {
      channel.send_state = SendState::Idle;
    });
  }

  /// Returns the amount of free space currently available in the
  /// channel.
  #[must_use]
  pub fn free_space(&self) -> usize {
    self.with_channel(|channel| {
      assert!(
        channel.queued_size <= channel.capacity,
        "Channel capacity unexpectedly exceeded",
      );
      channel.capacity - channel.queued_size
    })
  }

  /// Returns the terminal error, if one has occurred.
  #[must_use]
  pub fn error(&self) -> Option<ChannelError<E>> {
    self.with_channel(|channel| channel.error())
  }

  /// Reports an untyped terminal failure.
  ///
  /// The receiver drains messages already sent before it observes the
  /// abort.
  /// Further attempts to send also return the abort.
  /// Repeated calls have no effect.
  pub fn abort(&self) {
    self.abort_with(UnspecifiedChannelAbort);
  }

  /// Reports an untyped terminal failure caused by `error`.
  ///
  /// The receiver drains messages already sent before it observes the
  /// abort.
  /// Further attempts to send also return the abort.
  /// Repeated calls have no effect.
  pub fn abort_with(&self, error: impl Into<ChannelErrorInfo>) {
    self.with_channel_mut(|channel| {
      channel.terminate(
        TerminationReason::Aborted(error.into()),
        TerminatedBy::Transmitter,
      );
    });
  }

  /// Reports a terminal failure.
  ///
  /// The receiver drains messages already sent before it observes the
  /// failure. Further attempts to send also return the failure.
  /// Repeated calls have no effect.
  pub fn fail(&self, error: impl Into<ChannelErrorInfo<E>>) {
    self.with_channel_mut(|channel| {
      channel.terminate(
        TerminationReason::Failed(error.into()),
        TerminatedBy::Transmitter,
      );
    });
  }

  async fn send_helper(
    &self,
    mut message: T,
    latency: Duration,
    timeout: Option<Duration>,
  ) -> Result<(), SendError<T, E>> {
    self.declare_sender();
    let deadline = timeout.map(|timeout| Task::now() + timeout);
    let mut sent = 0_usize;
    loop {
      let options = SendOptions::new().latency(latency);
      let (unsent, newly_sent) =
        match self.try_send_with_options_helper(message, options) {
          Ok(result) => result,
          Err(mut error) => {
            error.sent = error
              .sent
              .checked_add(sent)
              .expect("Tx::send_with_options(): Sent size overflow.");
            return Err(error);
          }
        };
      sent = sent
        .checked_add(newly_sent)
        .expect("Tx::send_with_options(): Sent size overflow.");
      message = match unsent {
        None => return Ok(()),
        Some(message) => message,
      };
      if deadline.is_some_and(|deadline| deadline <= Task::now()) {
        return Err(SendError {
          cause: ChannelError::TimedOut,
          sent,
          unsent: message,
        });
      }
      self.with_channel_mut(|channel| {
        channel.enter_send_state(SendState::Waiting(Task::key()));
      });
      if let Some(deadline) = deadline {
        Task::sleep_until(deadline).await;
      } else {
        Task::park(|_| {}).await;
      }
      self.clear_send_state();
    }
  }

  /// Sends `message` with zero latency, using this endpoint's sending
  /// timeout.
  pub async fn send(&self, message: T) -> Result<(), SendError<T, E>> {
    self.send_with_options(message, SendOptions::new()).await
  }

  /// Sends `message` according to `options`.
  ///
  /// When space is insufficient, this function waits until space is
  /// available or the configured timeout expires.
  /// An option timeout overrides this endpoint's sending timeout.
  /// If neither is configured, there is no timeout.
  ///
  /// If the channel has a [`message_split()`] function, the message may
  /// be sent in multiple pieces as space becomes available.
  /// The configured latency is applied to each piece.
  ///
  /// If the timeout expires first, [`ChannelError::TimedOut`] is
  /// returned in a [`SendError`] with the sent size and unsent part of
  /// the message.
  ///
  /// [`ChannelError::TimedOut`]: crate::ChannelError::TimedOut
  /// [`message_split()`]: crate::Channel::message_split()
  pub async fn send_with_options(
    &self,
    message: T,
    options: SendOptions,
  ) -> Result<(), SendError<T, E>> {
    self
      .send_helper(
        message,
        options.latency,
        options.timeout.or(self.send_timeout.get()),
      )
      .await
  }

  /// Tries to send a message with zero latency without waiting.
  ///
  /// If there is some free space but not enough for the message, a
  /// [`message_split()`] function may split it to send a piece that
  /// fits.
  /// In that case, `Some` contains the remaining piece.
  /// Otherwise, the return value is `None` if the entire message was
  /// sent or `Some` containing the original message if none of it was
  /// sent.
  ///
  /// [`message_split()`]: crate::Channel::message_split()
  pub fn try_send(
    &self,
    message: T,
  ) -> Result<Option<T>, SendError<T, E>> {
    self.try_send_with_options(message, SendOptions::new())
  }

  /// Tries to send `message` according to `options` without waiting.
  ///
  /// The configured timeout is ignored because this function never
  /// waits.
  /// The configured latency is applied to any part of the message that
  /// is sent.
  ///
  /// If there is some free space but not enough for the message, a
  /// [`message_split()`] function may split it to send a piece that
  /// fits.
  /// In that case, `Some` contains the remaining piece.
  /// Otherwise, the return value is `None` if the entire message was
  /// sent or `Some` containing the original message if none of it was
  /// sent.
  ///
  /// [`message_split()`]: crate::Channel::message_split()
  pub fn try_send_with_options(
    &self,
    message: T,
    options: SendOptions,
  ) -> Result<Option<T>, SendError<T, E>> {
    self
      .try_send_with_options_helper(message, options)
      .map(|(unsent, _sent)| unsent)
  }

  fn try_send_with_options_helper(
    &self,
    mut message: T,
    options: SendOptions,
  ) -> Result<(Option<T>, usize), SendError<T, E>> {
    self.declare_sender();
    let latency = options.latency;
    self.with_channel_mut(|channel| {
      if let Some(error) = channel.error() {
        return Err(SendError {
          cause: error,
          sent: 0,
          unsent: message,
        });
      }
      if channel.local {
        assert!(
          latency == Duration::ZERO,
          "A send into a local channel must use zero latency"
        );
      }
      assert!(
        channel.queued_size <= channel.capacity,
        "Channel capacity unexpectedly exceeded"
      );
      let free = channel.capacity - channel.queued_size;
      let mut size = channel.get_message_size(&message);
      let mut rest = None;
      if size > free {
        let Some(message_split) = &channel.message_split else {
          assert!(
            size <= channel.capacity,
            "A message must not be impossibly large for a channel",
          );
          return Ok((Some(message), 0));
        };
        if free == 0 {
          return Ok((Some(message), 0));
        }
        let Some(tail) = message_split(&mut message, free) else {
          assert!(
            channel.queued_size > 0,
            "A message must not be impossibly large for a channel",
          );
          return Ok((Some(message), 0));
        };
        size = channel.get_message_size(&message);
        assert!(
          size > 0,
          "message_split must leave a nonempty message"
        );
        assert!(
          size <= free,
          "message_split must leave a message that fits"
        );
        rest = Some(tail);
      }
      let old_arrival = channel.queue.front_arrival();
      let arrival = channel
        .queue
        .push_back(Task::now() + latency, (message, size));
      channel.queued_size += size;
      if let Some(rx_task) = channel.rx_task
        && old_arrival.is_none_or(|old_arrival| arrival < old_arrival)
      {
        reschedule(rx_task, arrival);
      }
      Ok((rest, size))
    })
  }

  /// Equivalent to [`try_send()`], but panics if the entire message
  /// cannot be sent immediately.
  ///
  /// [`try_send()`]: crate::Tx::try_send()
  pub fn expect_send(&self, message: T) -> Result<(), SendError<T, E>> {
    let unsent = self.try_send(message)?;
    assert!(
      unsent.is_none(),
      "The channel must have enough free space to send the message",
    );
    Ok(())
  }

  //--------------------------------------------------------------------
  // Options
  //--------------------------------------------------------------------

  /// Returns the free-space threshold for write readiness.
  #[must_use]
  pub fn readiness_threshold(&self) -> usize {
    self.with_channel(|channel| channel.readiness_threshold)
  }

  /// Sets the free-space threshold for write readiness.
  ///
  /// The threshold must be positive.
  /// The default is one size unit.
  pub fn set_readiness_threshold(&self, threshold: usize) {
    assert!(
      threshold > 0,
      "netspin_core::Tx::set_readiness_threshold(): The threshold must \
       be positive.",
    );
    self.with_channel_mut(|channel| {
      channel.readiness_threshold = threshold;
    });
  }

  /// Returns the sending timeout.
  #[must_use]
  pub fn send_timeout(&self) -> Option<Duration> {
    self.send_timeout.get()
  }

  /// Sets the sending timeout.
  ///
  /// The default is no timeout.
  /// `None` or a zero duration means no timeout.
  pub fn set_send_timeout(&self, timeout: Option<Duration>) {
    self
      .send_timeout
      .set(timeout.filter(|timeout| !timeout.is_zero()));
  }
}

impl<T, E: Error + 'static> Pollable for Tx<T, E> {
  fn poll_target(&self) -> PollTarget<'_> {
    PollTarget::new(self)
  }
}

impl<T, E: Error + 'static> PollTargetOps for Tx<T, E> {
  fn declare_poller(&self) {
    self.declare_sender();
  }

  fn ready_at(&self) -> Option<Moment> {
    self.with_channel(|channel| {
      (channel.termination.is_some()
        || channel.capacity - channel.queued_size
          >= channel.readiness_threshold)
        .then_some(channel.tx_ready_at)
    })
  }

  fn set_poll_task(&self) {
    self.with_channel_mut(|channel| {
      channel.enter_send_state(SendState::Waiting(Task::key()));
    });
  }

  fn clear_poll_task(&self) {
    self.clear_send_state();
  }
}

impl<T, E: Error + 'static> Splice<Rx<T, E>> for Tx<T, E> {
  fn splice(mut self, mut upstream_rx: Rx<T, E>) {
    upstream_rx.assert_spliceable("netspin_core::Tx::splice()");
    let downstream_channel = self.channel();
    let upstream_channel = upstream_rx.channel_ref().clone();
    assert!(
      !Rc::ptr_eq(&downstream_channel, &upstream_channel),
      "Splicing channels must not create a cycle.",
    );
    {
      let downstream_channel = downstream_channel.borrow();
      let upstream_channel = upstream_channel.borrow();
      downstream_channel.assert_spliceable_downstream();
      upstream_channel.assert_spliceable();
    }
    let upstream_tx_channel =
      upstream_channel.borrow().tx_channel.upgrade().expect(
        "A spliceable channel must have a transmitting endpoint",
      );
    // Move the upstream state to the downstream receiver's allocation,
    // then retarget the upstream transmitter to that allocation.
    downstream_channel.swap(&upstream_channel);
    *upstream_tx_channel.borrow_mut() = downstream_channel;
    self.channel.take();
    upstream_rx.take_channel();
  }
}

impl<T, E: Error + 'static> Drop for Tx<T, E> {
  fn drop(&mut self) {
    let Some(channel) = &self.channel else {
      return;
    };
    let channel = channel.borrow();
    channel
      .borrow_mut()
      .terminate(TerminationReason::Closed, TerminatedBy::Transmitter);
  }
}

#[cfg(test)]
mod tests;
