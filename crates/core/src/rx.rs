//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::Cell;
use core::convert::Infallible;
use core::error::Error;
use core::time::Duration;
use std::rc::Rc;

use crate::Activity;
use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::ChannelReservation;
use crate::PollTarget;
use crate::Pollable;
use crate::ReceiveOptions;
use crate::Splice;
use crate::Tx;
use crate::channel_error::UnspecifiedChannelAbort;
use crate::channel_state::ChannelRef;
use crate::channel_state::ChannelState;
use crate::channel_state::TerminatedBy;
use crate::channel_state::TerminationReason;
use crate::moment::Moment;
use crate::poll::poll_until;
use crate::poll_target_ops::sealed::PollTargetOps;
use crate::task::Task;
use crate::task::TaskId;

type ReceiveWork<T> = (Activity, Rc<dyn Fn(&T) -> Duration>);

/// The receiving endpoint of a [`Channel`].
///
/// The first task to receive from or poll an [`Rx`] endpoint becomes
/// its consumer task.
/// All subsequent receive and poll operations must be performed by the
/// consumer task.
///
/// [`Channel`]: crate::Channel
/// [`Rx`]: crate::Rx
#[must_use]
pub struct Rx<T, E: Error + 'static = Infallible> {
  channel: Option<ChannelRef<T, E>>,
  receive_work: Option<ReceiveWork<T>>,
  task_id: Cell<Option<TaskId>>,

  //--------------------------------------------------------------------
  // Options
  //--------------------------------------------------------------------
  receive_timeout: Cell<Option<Duration>>,
}

impl<T, E: Error + 'static> Rx<T, E> {
  pub(super) fn new(channel: ChannelRef<T, E>) -> Self {
    Self {
      channel: Some(channel),
      receive_work: None,
      task_id: Cell::new(None),
      receive_timeout: Cell::new(None),
    }
  }

  fn channel(&self) -> ChannelRef<T, E> {
    self.channel_ref().clone()
  }

  pub(super) fn channel_ref(&self) -> &ChannelRef<T, E> {
    self.channel.as_ref().unwrap()
  }

  fn with_channel<R>(
    &self,
    f: impl FnOnce(&ChannelState<T, E>) -> R,
  ) -> R {
    f(&self.channel_ref().borrow())
  }

  fn with_channel_mut<R>(
    &self,
    f: impl FnOnce(&mut ChannelState<T, E>) -> R,
  ) -> R {
    f(&mut self.channel_ref().borrow_mut())
  }

  pub(super) fn assert_spliceable(&self, function: &str) {
    assert!(
      self.receive_work.is_none(),
      "{function}: A receiver with modeled receive work must not be \
       spliced.",
    );
  }

  fn assert_no_receive_work(&self, function: &str) {
    assert!(
      self.receive_work.is_none(),
      "{function}: Modeled receive work requires an asynchronous \
       receive.",
    );
  }

  pub(super) fn take_channel(&mut self) {
    self.channel.take();
  }

  fn try_receive_helper(
    &self,
    max_size: Option<usize>,
  ) -> Result<Option<T>, ChannelError<E>> {
    self.declare_poller();
    let clock = Task::now();
    self.with_channel_mut(|channel| {
      if let Some(error) = channel.receiver_error() {
        return Err(error);
      }
      if channel
        .queue
        .front_arrival()
        .is_some_and(|arrival| arrival <= clock)
      {
        let (key, (mut message, size)) =
          channel.queue.pop_first().unwrap();
        if let Some(max_size) = max_size
          && size > max_size
        {
          let message_split =
            channel.message_split.clone().unwrap_or_else(|| {
              panic!(
                "netspin_core::Rx::try_receive_up_to(): The ready \
                 message exceeds the maximum size, but the channel \
                 has no message splitter.",
              )
            });
          let tail = message_split(&mut message, max_size)
            .unwrap_or_else(|| {
              panic!(
                "netspin_core::Rx::try_receive_up_to(): The channel's \
                 message splitter declined the required split.",
              )
            });
          let prefix_size = channel.get_message_size(&message);
          let tail_size = channel.get_message_size(&tail);
          assert!(
            prefix_size > 0,
            "netspin_core::Rx::try_receive_up_to(): The channel's \
             message splitter produced an empty prefix.",
          );
          assert!(
            prefix_size <= max_size,
            "netspin_core::Rx::try_receive_up_to(): The channel's \
             message splitter produced an oversized prefix.",
          );
          assert!(
            tail_size > 0,
            "netspin_core::Rx::try_receive_up_to(): The channel's \
             message splitter produced an empty suffix.",
          );
          channel.queued_size = channel
            .queued_size
            .checked_sub(size)
            .and_then(|size| size.checked_add(tail_size))
            .expect(
              "netspin_core::Rx::try_receive_up_to(): The queued size \
               overflowed.",
            );
          assert!(
            channel.queued_size <= channel.capacity,
            "netspin_core::Rx::try_receive_up_to(): The retained suffix \
             exceeds the channel capacity.",
          );
          channel.queue.push_front(key, (tail, tail_size));
        } else {
          channel.queued_size -= size;
        }
        channel.interrupt_sender();
        return Ok(Some(message));
      }
      if let Some(error) = channel.receiver_error() {
        return Err(error);
      }
      Ok(None)
    })
  }

  async fn receive_helper(
    &self,
    timeout: Option<Duration>,
  ) -> Result<T, ChannelError<E>> {
    self.declare_poller();
    let deadline = timeout.map(|timeout| Task::now() + timeout);
    poll_until(&[self], deadline).await;
    match self.try_receive_helper(None)? {
      Some(message) => {
        if let Some((activity, work)) = &self.receive_work {
          let duration = work(&message);
          if !duration.is_zero() {
            let lease = activity.lease();
            Task::work(duration).await;
            drop(lease);
          }
        }
        Ok(message)
      }
      None => {
        assert!(
          timeout.is_some(),
          "Rx::receive_with_options(): A receive without a timeout \
           must not return without a message.",
        );
        Err(ChannelError::TimedOut)
      }
    }
  }

  /// Receives the next message, using this endpoint's receiving
  /// timeout.
  pub async fn receive(&self) -> Result<T, ChannelError<E>> {
    self.receive_with_options(ReceiveOptions::new()).await
  }

  /// Receives the next message according to `options`.
  ///
  /// An option timeout overrides this endpoint's receiving timeout.
  /// [`ChannelError::TimedOut`] is returned if the effective timeout
  /// expires before a message is ready.
  /// If neither is configured, there is no timeout.
  /// If the channel is ready at the same moment the timeout expires,
  /// readiness wins.
  ///
  /// [`ChannelError::TimedOut`]: crate::ChannelError::TimedOut
  pub async fn receive_with_options(
    &self,
    options: ReceiveOptions,
  ) -> Result<T, ChannelError<E>> {
    self
      .receive_helper(options.timeout.or(self.receive_timeout.get()))
      .await
  }

  /// Tries to receive the next ready message without waiting.
  ///
  /// The return value is `None` if no message is ready.
  pub fn try_receive(&self) -> Result<Option<T>, ChannelError<E>> {
    self.try_receive_with_options(ReceiveOptions::new())
  }

  /// Tries to receive at most `max_size` size units from the next
  /// ready message without waiting.
  ///
  /// Message size is determined by the channel's
  /// [`message_size()`] function.
  /// If the message exceeds `max_size`, the channel's
  /// [`message_split()`] function splits it.
  /// The returned prefix is removed from the channel, while the suffix
  /// retains the original message's arrival moment and ordering
  /// position.
  ///
  /// This function never combines separate messages.
  /// The return value is `None` only if no message is ready.
  ///
  /// Panics if `max_size` is zero, or if a ready message exceeds it and
  /// the channel has no message-split function, the function declines
  /// the split, or the split produces an invalid result.
  ///
  /// [`message_size()`]: crate::Channel::message_size()
  /// [`message_split()`]: crate::Channel::message_split()
  pub fn try_receive_up_to(
    &self,
    max_size: usize,
  ) -> Result<Option<T>, ChannelError<E>> {
    assert!(
      max_size > 0,
      "netspin_core::Rx::try_receive_up_to(): The maximum size must be \
       positive.",
    );
    self
      .assert_no_receive_work("netspin_core::Rx::try_receive_up_to()");
    self.try_receive_helper(Some(max_size))
  }

  /// Tries to receive the next ready message according to `options`
  /// without waiting.
  ///
  /// The configured timeout is ignored because this function never
  /// waits.
  /// The return value is `None` if no message is ready.
  pub fn try_receive_with_options(
    &self,
    _options: ReceiveOptions,
  ) -> Result<Option<T>, ChannelError<E>> {
    self.assert_no_receive_work(
      "netspin_core::Rx::try_receive_with_options()",
    );
    self.try_receive_helper(None)
  }

  /// Tries to receive the next ready message while retaining its
  /// channel space.
  #[allow(clippy::type_complexity)]
  pub fn try_receive_reserved(
    &self,
  ) -> Result<Option<(T, ChannelReservation<T, E>)>, ChannelError<E>>
  {
    self.assert_no_receive_work(
      "netspin_core::Rx::try_receive_reserved()",
    );
    self.declare_poller();
    let clock = Task::now();
    let channel = self.channel();
    let message = {
      let mut channel = channel.borrow_mut();
      if let Some(error) = channel.receiver_error() {
        return Err(error);
      }
      if channel
        .queue
        .front_arrival()
        .is_some_and(|arrival| arrival <= clock)
      {
        let (_, (message, size)) = channel.queue.pop_first().unwrap();
        Ok(Some((message, size)))
      } else {
        if let Some(error) = channel.receiver_error() {
          Err(error)
        } else {
          Ok(None)
        }
      }
    }?;
    Ok(message.map(|(message, size)| {
      (message, ChannelReservation::new(channel.clone(), size))
    }))
  }

  /// Equivalent to [`try_receive()`], but panics if there are no
  /// messages ready to receive.
  ///
  /// [`try_receive()`]: Rx::try_receive()
  pub fn expect_receive(&self) -> Result<T, ChannelError<E>> {
    self.declare_poller();
    Ok(
      self
        .try_receive()?
        .expect("The channel must have a message ready to receive"),
    )
  }

  /// Equivalent to [`try_receive_reserved()`], but panics if there are
  /// no messages ready to receive.
  ///
  /// [`try_receive_reserved()`]: Rx::try_receive_reserved()
  pub fn expect_receive_reserved(
    &self,
  ) -> Result<(T, ChannelReservation<T, E>), ChannelError<E>> {
    self.declare_poller();
    Ok(
      self
        .try_receive_reserved()?
        .expect("The channel must have a message ready to receive"),
    )
  }

  /// Returns whether the channel contains no queued messages.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.with_channel(|channel| channel.queue.is_empty())
  }

  /// Reports an untyped terminal failure.
  ///
  /// The abort is immediately visible to the transmitter and this
  /// receiver.
  /// Queued messages are discarded.
  /// Repeated calls have no effect.
  pub fn abort(&self) {
    self.abort_with(UnspecifiedChannelAbort);
  }

  /// Reports an untyped terminal failure caused by `error`.
  ///
  /// The abort is immediately visible to the transmitter and this
  /// receiver.
  /// Queued messages are discarded.
  /// Repeated calls have no effect.
  pub fn abort_with(&self, error: impl Into<ChannelErrorInfo>) {
    self.with_channel_mut(|channel| {
      channel.terminate(
        TerminationReason::Aborted(error.into()),
        TerminatedBy::Receiver,
      );
    });
  }

  /// Reports a terminal failure.
  ///
  /// The failure is immediately visible to the transmitter and this
  /// receiver. Queued messages are discarded. Repeated calls have no
  /// effect.
  pub fn fail(&self, error: impl Into<ChannelErrorInfo<E>>) {
    self.with_channel_mut(|channel| {
      channel.terminate(
        TerminationReason::Failed(error.into()),
        TerminatedBy::Receiver,
      );
    });
  }

  //--------------------------------------------------------------------
  // Options
  //--------------------------------------------------------------------

  /// Returns the receiving timeout.
  #[must_use]
  pub fn receive_timeout(&self) -> Option<Duration> {
    self.receive_timeout.get()
  }

  /// Sets the receiving timeout.
  ///
  /// The default is no timeout.
  /// `None` or a zero duration means no timeout.
  pub fn set_receive_timeout(&self, timeout: Option<Duration>) {
    self
      .receive_timeout
      .set(timeout.filter(|timeout| !timeout.is_zero()));
  }

  /// Sets the active work performed after each successful asynchronous
  /// receive and the activity held while that work is performed.
  ///
  /// The work function receives the message and returns its modeled
  /// service duration.
  /// A zero duration performs no work and does not acquire the
  /// activity.
  /// The default is no modeled receive work.
  ///
  /// Only [`receive()`] and [`receive_with_options()`] can suspend to
  /// perform work.
  /// Configuring work therefore makes the synchronous receive methods
  /// unavailable.
  ///
  /// [`receive()`]: Rx::receive()
  /// [`receive_with_options()`]: Rx::receive_with_options()
  pub fn work_on_receive(
    mut self,
    activity: Activity,
    work: impl Fn(&T) -> Duration + 'static,
  ) -> Self {
    self.receive_work = Some((activity, Rc::new(work)));
    self
  }
}

impl<T, E: Error + 'static> Splice<Tx<T, E>> for Rx<T, E> {
  fn splice(self, downstream_tx: Tx<T, E>) {
    self.assert_spliceable("netspin_core::Rx::splice()");
    downstream_tx.splice(self);
  }
}

impl<T, E: Error + 'static> Pollable for Rx<T, E> {
  fn poll_target(&self) -> PollTarget<'_> {
    PollTarget::new(self)
  }
}

impl<T, E: Error + 'static> PollTargetOps for Rx<T, E> {
  fn declare_poller(&self) {
    if cfg!(debug_assertions) {
      if let Some(id) = self.task_id.get() {
        debug_assert!(
          Task::id() == id,
          "A channel must not have multiple receivers",
        );
      } else {
        self.task_id.set(Some(Task::id()));
      }
    }
  }

  fn ready_at(&self) -> Option<Moment> {
    self.with_channel(|channel| channel.front_arrival())
  }

  fn set_poll_task(&self) {
    self.with_channel_mut(|channel| {
      channel.rx_task = Some(Task::key());
    });
  }

  fn clear_poll_task(&self) {
    self.with_channel_mut(|channel| {
      channel.rx_task = None;
    });
  }
}

impl<T, E: Error + 'static> Drop for Rx<T, E> {
  fn drop(&mut self) {
    let Some(channel) = &self.channel else {
      return;
    };
    channel
      .borrow_mut()
      .terminate(TerminationReason::Closed, TerminatedBy::Receiver);
  }
}

#[cfg(test)]
mod tests;
