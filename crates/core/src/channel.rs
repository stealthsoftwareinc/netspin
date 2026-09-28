//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cell::RefCell;
use core::convert::Infallible;
use core::error::Error;
use core::fmt;
use core::marker::PhantomData;
use std::rc::Rc;

use crate::channel_state::ChannelState;
use crate::rx::Rx;
use crate::tx::Tx;

pub(crate) type MessageSizer<T> = dyn Fn(&T) -> usize;
pub(crate) type MessageSplitter<T> = dyn Fn(&mut T, usize) -> Option<T>;

/// The message ordering behavior of a channel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChannelOrder {
  /// Messages arrive in the same order they were sent, regardless of
  /// latency.
  ///
  /// A low-latency message sent after a high-latency message will catch
  /// up to and ride behind it instead of overtaking it.
  #[default]
  Ordered,

  /// Messages arrive at exactly their send time plus their latency,
  /// regardless of the order they were sent in.
  ///
  /// A low-latency message sent after a high-latency message will
  /// overtake it.
  Unordered,
}

/// Configures a channel.
///
/// The default channel settings are:
///
/// * [`capacity()`][]: 1.
/// * [`local()`][]: `true`.
/// * [`message_size()`][]: None.
/// * [`message_split()`][]: None.
/// * [`order()`][]: [`Ordered`].
///
/// [`Ordered`]: crate::ChannelOrder::Ordered
/// [`capacity()`]: crate::Channel::capacity()
/// [`local()`]: crate::Channel::local()
/// [`message_size()`]: crate::Channel::message_size()
/// [`message_split()`]: crate::Channel::message_split()
/// [`order()`]: crate::Channel::order()
pub struct Channel<T, E: Error + 'static = Infallible> {
  pub(crate) capacity: Option<usize>,

  pub(crate) message_size: Option<Rc<MessageSizer<T>>>,

  pub(crate) message_split: Option<Rc<MessageSplitter<T>>>,

  pub(crate) local: bool,

  pub(crate) order: ChannelOrder,

  error: PhantomData<fn() -> E>,
}

impl<T, E: Error + 'static> Clone for Channel<T, E> {
  fn clone(&self) -> Self {
    Self {
      capacity: self.capacity,
      message_size: self.message_size.clone(),
      message_split: self.message_split.clone(),
      local: self.local,
      order: self.order,
      error: PhantomData,
    }
  }
}

impl<T, E: Error + 'static> fmt::Debug for Channel<T, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Channel")
      .field("capacity", &self.capacity)
      .field("local", &self.local)
      .field("order", &self.order)
      .finish_non_exhaustive()
  }
}

impl<T, E: Error + 'static> Channel<T, E> {
  /// Sets the maximum permitted total message size.
  ///
  /// `None` makes the channel unbounded.
  ///
  /// The default is `1`.
  #[must_use]
  pub fn capacity(mut self, capacity: Option<usize>) -> Self {
    self.capacity = capacity;
    self
  }

  /// Sets whether the channel is local.
  ///
  /// A local channel is a channel in which messages should always be
  /// sent with zero latency.
  /// Setting this option makes [`send_with_options()`] and
  /// [`try_send_with_options()`] assert that the configured latency is
  /// zero.
  ///
  /// The default is `true`.
  ///
  /// [`send_with_options()`]: crate::Tx::send_with_options()
  /// [`try_send_with_options()`]: crate::Tx::try_send_with_options()
  #[must_use]
  pub fn local(mut self, local: bool) -> Self {
    self.local = local;
    self
  }

  /// Sets the function that computes the size of a message.
  ///
  /// The default is a function that returns `1` for every message.
  #[must_use]
  pub fn message_size(
    mut self,
    message_size: impl Fn(&T) -> usize + 'static,
  ) -> Self {
    self.message_size = Some(Rc::new(message_size));
    self
  }

  /// Sets the function that splits a message at a size limit.
  ///
  /// [`Tx`] uses this function when a channel has nonzero free space
  /// that is insufficient for a sent message.
  /// [`Rx::try_receive_up_to()`] uses it when a ready message exceeds
  /// the requested maximum.
  ///
  /// The function is called with a reference to the message and the
  /// size limit.
  /// The function should either truncate the message so its size
  /// ([`message_size()`]) is positive and at most the limit, returning
  /// the nonempty suffix that was split off, or return `None` to
  /// decline the split.
  /// The two pieces resulting from the split need not have sizes that
  /// sum to the size of the original message, as splitting may incur
  /// extra framing.
  ///
  /// The default is to never split messages.
  ///
  /// [`Rx`]: crate::Rx
  /// [`Rx::try_receive_up_to()`]: crate::Rx::try_receive_up_to()
  /// [`Tx`]: crate::Tx
  /// [`message_size()`]: Channel::message_size()
  #[must_use]
  pub fn message_split(
    mut self,
    message_split: impl Fn(&mut T, usize) -> Option<T> + 'static,
  ) -> Self {
    self.message_split = Some(Rc::new(message_split));
    self
  }

  /// Sets the message ordering behavior.
  ///
  /// The default is [`ChannelOrder::Ordered`].
  ///
  /// [`ChannelOrder::Ordered`]: crate::ChannelOrder::Ordered
  #[must_use]
  pub fn order(mut self, order: ChannelOrder) -> Self {
    self.order = order;
    self
  }

  /// Creates the channel, returning its sender and receiver.
  pub fn pair(self) -> (Tx<T, E>, Rx<T, E>) {
    let channel = Rc::new(RefCell::new(ChannelState::new(self)));
    let tx_channel = Rc::new(RefCell::new(channel.clone()));
    channel.borrow_mut().tx_channel = Rc::downgrade(&tx_channel);
    let tx = Tx::new(tx_channel);
    let rx = Rx::new(channel);
    (tx, rx)
  }
}

impl<T, E: Error + 'static> Default for Channel<T, E> {
  fn default() -> Self {
    Self {
      capacity: Some(1),
      message_size: None,
      message_split: None,
      local: true,
      order: ChannelOrder::default(),
      error: PhantomData,
    }
  }
}

impl<T> Channel<T> {
  /// Creates a channel with the default settings.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Selects a custom failure type for this channel.
  ///
  /// Untyped aborts are reported as [`ChannelError::Aborted`].
  /// Dropping an endpoint is reported as [`ChannelError::Closed`].
  /// Explicit failures are reported as [`ChannelError::Failed`].
  ///
  /// [`ChannelError::Aborted`]: crate::ChannelError::Aborted
  /// [`ChannelError::Closed`]: crate::ChannelError::Closed
  /// [`ChannelError::Failed`]: crate::ChannelError::Failed
  #[must_use]
  pub fn fallible<E: Error + 'static>(self) -> Channel<T, E> {
    let Self {
      capacity,
      message_size,
      message_split,
      local,
      order,
      error: _,
    } = self;
    Channel {
      capacity,
      message_size,
      message_split,
      local,
      order,
      error: PhantomData,
    }
  }
}

#[cfg(test)]
mod tests;
