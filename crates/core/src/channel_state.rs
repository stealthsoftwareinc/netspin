//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::error::Error;
use std::rc::Rc;
use std::rc::Weak;

use crate::ChannelError;
use crate::ChannelErrorInfo;
use crate::channel::Channel;
use crate::channel::MessageSizer;
use crate::channel::MessageSplitter;
use crate::channel_queue::ChannelQueue;
use crate::executor::reschedule;
use crate::moment::Moment;
use crate::task::Task;
use crate::task_key::TaskKey;

pub(super) type ChannelRef<T, E> = Rc<RefCell<ChannelState<T, E>>>;
pub(super) type TxChannelRef<T, E> = Rc<RefCell<ChannelRef<T, E>>>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum TerminatedBy {
  Receiver,
  Transmitter,
}

pub(super) struct ChannelTermination<E: Error + 'static> {
  pub(super) reason: TerminationReason<E>,
  pub(super) moment: Moment,
  pub(super) terminated_by: TerminatedBy,
}

pub(super) enum TerminationReason<E: Error + 'static> {
  Aborted(ChannelErrorInfo),
  Closed,
  Failed(ChannelErrorInfo<E>),
}

impl<E: Error + 'static> Clone for TerminationReason<E> {
  fn clone(&self) -> Self {
    match self {
      Self::Aborted(info) => Self::Aborted(info.clone()),
      Self::Closed => Self::Closed,
      Self::Failed(info) => Self::Failed(info.clone()),
    }
  }
}

impl<E: Error + 'static> TerminationReason<E> {
  fn error(&self) -> ChannelError<E> {
    match self {
      Self::Aborted(info) => ChannelError::Aborted(info.clone()),
      Self::Closed => ChannelError::Closed,
      Self::Failed(info) => ChannelError::Failed(info.clone()),
    }
  }
}

pub(super) enum SendState {
  Idle,
  Waiting(TaskKey),
}

pub(super) struct ChannelState<T, E: Error + 'static> {
  /// The queued messages, each stored along with its size.
  pub(super) queue: ChannelQueue<T>,

  /// The maximum permitted total message size of the queue.
  pub(super) capacity: usize,

  /// The free-space threshold at which the transmitter is ready.
  pub(super) readiness_threshold: usize,

  /// The function that computes the size of a message.
  pub(super) message_size: Option<Rc<MessageSizer<T>>>,

  /// The function that splits a message at a size limit.
  pub(super) message_split: Option<Rc<MessageSplitter<T>>>,

  /// The current total message size of the queue.
  pub(super) queued_size: usize,

  /// Whether the channel is local.
  ///
  /// A local channel verifies that all message sends use zero latency.
  pub(super) local: bool,

  /// How the task waiting to send the next message is suspended.
  pub(super) send_state: SendState,

  /// A weak reference to the transmitting endpoint's channel handle.
  ///
  /// A splice retargets this handle to the surviving receiver's
  /// channel state.
  pub(super) tx_channel: Weak<RefCell<ChannelRef<T, E>>>,

  /// The clock value at which a waiting sender became ready.
  pub(super) tx_ready_at: Moment,

  /// The receiver task, but only while it is waiting in a
  /// `receive()` or `poll()` call and must be rescheduled when a send
  /// creates an earlier front arrival.
  pub(super) rx_task: Option<TaskKey>,

  /// The first terminal event reported for the channel.
  pub(super) termination: Option<ChannelTermination<E>>,
}

impl<T, E: Error + 'static> ChannelState<T, E> {
  pub(super) fn new(channel: Channel<T, E>) -> Self {
    let Channel {
      capacity,
      message_size,
      message_split,
      local,
      order,
      ..
    } = channel;
    let capacity = capacity.unwrap_or(usize::MAX);
    assert!(capacity > 0, "Channel capacity must be positive");
    let single_message_queue = capacity == 1 && message_size.is_none();
    let queue = ChannelQueue::new(single_message_queue, order);
    Self {
      queue,
      capacity,
      readiness_threshold: 1,
      message_size,
      message_split,
      queued_size: 0,
      local,
      send_state: SendState::Idle,
      tx_channel: Weak::new(),
      tx_ready_at: Moment::ZERO,
      rx_task: None,
      termination: None,
    }
  }

  pub(super) fn enter_send_state(&mut self, state: SendState) {
    debug_assert!(
      match (&self.send_state, &state) {
        (SendState::Idle, _) => true,
        (SendState::Waiting(old), SendState::Waiting(new)) => {
          old == new
        }
        _ => false,
      },
      "Channels must not have multiple senders",
    );
    self.send_state = state;
  }

  pub(super) fn get_message_size(&self, message: &T) -> usize {
    self
      .message_size
      .as_ref()
      .map_or(1, |message_size| message_size(message))
  }

  pub(super) fn interrupt_sender(&mut self) {
    if self.termination.is_none()
      && self.capacity - self.queued_size < self.readiness_threshold
    {
      return;
    }
    let send_state =
      core::mem::replace(&mut self.send_state, SendState::Idle);
    if !Task::is_running() {
      return;
    }
    let now = Task::now();
    match send_state {
      SendState::Idle => {}
      SendState::Waiting(task_id) => {
        self.tx_ready_at = now;
        reschedule(task_id, now);
      }
    }
  }

  fn interrupt_receiver(&mut self, arrival: Moment) {
    let current_moment = Task::try_now();
    let rx_task = self.rx_task.take();
    if current_moment.is_some()
      && let Some(task_id) = rx_task
    {
      reschedule(task_id, arrival);
    }
  }

  pub(super) fn terminate(
    &mut self,
    reason: TerminationReason<E>,
    terminated_by: TerminatedBy,
  ) {
    if self.termination.is_some() {
      return;
    }
    let moment = Task::try_now().unwrap_or(Moment::ZERO);
    if terminated_by == TerminatedBy::Receiver {
      let queued_size = self.queue.drain_size();
      self.queued_size -= queued_size;
    }
    self.termination = Some(ChannelTermination {
      reason,
      moment,
      terminated_by,
    });
    self.interrupt_sender();
    let arrival = self.front_arrival().unwrap();
    self.interrupt_receiver(arrival);
  }

  pub(super) fn error(&self) -> Option<ChannelError<E>> {
    self
      .termination
      .as_ref()
      .map(|termination| termination.reason.error())
  }

  pub(super) fn receiver_error(&self) -> Option<ChannelError<E>> {
    let termination = self.termination.as_ref()?;
    if termination.terminated_by == TerminatedBy::Receiver
      || self.queue.is_empty()
    {
      Some(termination.reason.error())
    } else {
      None
    }
  }

  pub(super) fn front_arrival(&self) -> Option<Moment> {
    let message = self.queue.front_arrival();
    let Some(termination) = &self.termination else {
      return message;
    };
    match termination.terminated_by {
      TerminatedBy::Receiver => Some(termination.moment),
      TerminatedBy::Transmitter => message.or(Some(termination.moment)),
    }
  }

  pub(super) fn assert_spliceable(&self) {
    assert!(
      self.queue.is_empty() && self.queued_size == 0,
      "Splicing a channel requires it to be empty.",
    );
    assert!(
      matches!(&self.send_state, SendState::Idle),
      "Splicing a channel requires it to have no active senders.",
    );
    assert!(
      self.rx_task.is_none(),
      "Splicing a channel requires it to have no active receivers.",
    );
    assert!(
      self.termination.is_none(),
      "Splicing a channel requires it to be open.",
    );
  }

  pub(super) fn assert_spliceable_downstream(&self) {
    self.assert_spliceable();
    assert!(self.local, "Splicing a channel requires it to be local.");
    assert!(
      self.message_size.is_none(),
      "Splicing a channel requires it to have no message size function.",
    );
    assert!(
      self.message_split.is_none(),
      "Splicing a channel requires it to have no message split function.",
    );
  }
}
