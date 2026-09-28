//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::cmp::Ordering;

use std::collections::BinaryHeap;
use std::collections::VecDeque;

use crate::ChannelOrder;
use crate::Moment;

type Key = (Moment, u64);
type Value<T> = (T, usize);

pub(super) struct QueuedMessage<T> {
  key: Key,
  value: Value<T>,
}

impl<T> Eq for QueuedMessage<T> {}

impl<T> Ord for QueuedMessage<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    // BinaryHeap pops its greatest entry, so reverse the key order.
    other.key.cmp(&self.key)
  }
}

impl<T> PartialEq for QueuedMessage<T> {
  fn eq(&self, other: &Self) -> bool {
    self.key == other.key
  }
}

impl<T> PartialOrd for QueuedMessage<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

/// Stores single-message channel queues inline.
/// Ordered queues use a FIFO because arrivals never move backward.
/// Unordered queues use a priority queue.
pub(super) enum ChannelQueue<T> {
  Ordered {
    messages: VecDeque<(Moment, Value<T>)>,
    min_arrival: Moment,
  },
  Single(Option<(Moment, Value<T>)>),
  Unordered {
    messages: BinaryHeap<QueuedMessage<T>>,
    next_id: u64,
  },
}

impl<T> ChannelQueue<T> {
  pub(super) fn new(
    single_message_queue: bool,
    order: ChannelOrder,
  ) -> Self {
    if single_message_queue {
      Self::Single(None)
    } else if order == ChannelOrder::Ordered {
      Self::Ordered {
        messages: VecDeque::new(),
        min_arrival: Moment::ZERO,
      }
    } else {
      Self::Unordered {
        messages: BinaryHeap::new(),
        next_id: 0,
      }
    }
  }

  pub(super) fn drain_size(&mut self) -> usize {
    match self {
      Self::Ordered { messages, .. } => {
        messages.drain(..).map(|(_, (_, size))| size).sum()
      }
      Self::Single(message) => {
        message.take().map_or(0, |(_, (_, size))| size)
      }
      Self::Unordered { messages, .. } => {
        messages.drain().map(|message| message.value.1).sum()
      }
    }
  }

  pub(super) fn front_arrival(&self) -> Option<Moment> {
    match self {
      Self::Ordered { messages, .. } => {
        messages.front().map(|(arrival, _)| *arrival)
      }
      Self::Single(message) => {
        message.as_ref().map(|(arrival, _)| *arrival)
      }
      Self::Unordered { messages, .. } => {
        messages.peek().map(|message| message.key.0)
      }
    }
  }

  pub(super) fn push_back(
    &mut self,
    arrival: Moment,
    value: Value<T>,
  ) -> Moment {
    match self {
      Self::Ordered {
        messages,
        min_arrival,
      } => {
        let arrival = arrival.max(*min_arrival);
        *min_arrival = arrival;
        debug_assert!(
          messages
            .back()
            .is_none_or(|(last_arrival, _)| *last_arrival <= arrival),
          "netspin_core::channel_queue::ChannelQueue::push_back(): \
           Message arrivals must not decrease.",
        );
        messages.push_back((arrival, value));
        arrival
      }
      Self::Single(message) => {
        assert!(
          message.is_none(),
          "netspin_core::channel_queue::ChannelQueue::push_back(): \
           A single-message queue must be empty.",
        );
        *message = Some((arrival, value));
        arrival
      }
      Self::Unordered { messages, next_id } => {
        let key = (arrival, *next_id);
        *next_id = next_id.checked_add(1).expect(
          "netspin_core::channel_queue::ChannelQueue::push_back(): \
           Message ID overflow.",
        );
        messages.push(QueuedMessage { key, value });
        arrival
      }
    }
  }

  pub(super) fn push_front(&mut self, key: Key, value: Value<T>) {
    match self {
      Self::Ordered { messages, .. } => {
        let arrival = key.0;
        debug_assert!(
          messages
            .front()
            .is_none_or(|(first_arrival, _)| arrival <= *first_arrival),
          "netspin_core::channel_queue::ChannelQueue::push_front(): \
           Message arrivals must not increase.",
        );
        messages.push_front((arrival, value));
      }
      Self::Single(message) => {
        assert!(
          message.is_none(),
          "netspin_core::channel_queue::ChannelQueue::push_front(): \
           A single-message queue must be empty.",
        );
        *message = Some((key.0, value));
      }
      Self::Unordered { messages, .. } => {
        messages.push(QueuedMessage { key, value });
      }
    }
  }

  pub(super) fn is_empty(&self) -> bool {
    match self {
      Self::Ordered { messages, .. } => messages.is_empty(),
      Self::Single(message) => message.is_none(),
      Self::Unordered { messages, .. } => messages.is_empty(),
    }
  }

  pub(super) fn pop_first(&mut self) -> Option<(Key, Value<T>)> {
    match self {
      Self::Ordered { messages, .. } => messages
        .pop_front()
        .map(|(arrival, value)| ((arrival, 0), value)),
      Self::Single(message) => {
        message.take().map(|(arrival, value)| ((arrival, 0), value))
      }
      Self::Unordered { messages, .. } => {
        messages.pop().map(|message| (message.key, message.value))
      }
    }
  }
}
