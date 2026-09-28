//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

//! A high-fidelity sampling-based network modeling framework.
//!
//! A suspended task is said to be *parked* if it can only be resumed by
//! the action of another task, not by the passage of time.
//! For example, a task awaiting a [`Tx::send()`] call on a channel with
//! no send timeout and insufficient space is parked.
//!
//! A suspended task that is not parked is said to be *unparked*,
//! meaning it will eventually be resumed if enough time passes.
//! For example, a task awaiting a [`Tx::send()`] call on a channel with
//! a send timeout is unparked.
//!
//! All panics produced by this library are fatal to the library.
//! Using the library after catching a library panic is not supported.
//!
//! ## Async rules
//!
//! NetSpin uses a custom async executor that cannot run arbitrary async
//! code.
//! When using the executor, all task suspensions must ultimately come
//! from awaiting exactly one NetSpin operation.
//! Functions awaiting other functions is fine, but you cannot use any
//! async utilities from other libraries.
//! The executor does its best to detect this and panic, but perfect
//! detection is not possible.
//!
//! [`Task::block_on()`] can be used to run a contained asynchronous
//! computation to completion without suspending the calling task.
//! The computation must not await NetSpin operations or depend on
//! another task on the calling executor making progress.
//!
//! ## Channels
//!
//! A *local* channel is a channel in which messages should always be
//! sent with zero latency.
//!
//! ## Blocks
//!
//! A *block* is a group of tasks and channels that work together to
//! model a component in the overall system.
//! A block is created with its `new()` function, configured with fluent
//! setters, and attached to an existing *target* set of primitives with
//! [`Attach::attach()`], which returns a *surface* set of primitives to
//! which more blocks can be attached.
//! This provides a modular approach to building complex topologies.
//!
//! Unless otherwise noted, all [`Tx`] and [`Rx`] endpoints returned by
//! blocks have local underlying channels with fast backpressure (i.e.,
//! a small capacity).
//! These are called *handoff* endpoints.
//! If you're attaching to these endpoints and you need more complex
//! behavior, you should use additional blocks or tasks instead of
//! trying to force the behavior onto the endpoints themselves.
//!
//! ## Connecting channels
//!
//! The [`Rx`] and [`Tx`] endpoints of two channels with the same
//! message type can be connected in two ways: by pumping or splicing.
//!
//! Pumping means creating a task that receives messages from the [`Rx`]
//! endpoint and sends them to the [`Tx`] endpoint.
//! This can be done with the [`Pump`] and [`DuplexPump`] blocks, or by
//! implementing a custom task that may have custom behavior but still
//! ultimately pumps messages from one endpoint to the other.
//! Pumping always increases the effective buffer space between the
//! original [`Rx`] and [`Tx`] endpoints, as the pump must always at
//! least hold (part of) one message in memory.
//!
//! Splicing removes the receiving endpoint of an upstream channel and
//! the transmitting endpoint of a downstream channel.
//! This leaves the remaining endpoints directly connected without a
//! forwarding task.
//! The upstream channel's queue and configuration survive, including
//! its capacity, while the downstream channel's queue is removed.
//! For example:
//!
//! ```text
//! Before: tx1 -> [Q1, C1] -> rx1 | tx2 -> [Q2, C2] -> rx2
//!                                  ^ splice here
//!
//! After:  tx1 -> [Q1, C1] ---------------------------> rx2
//! ```
//! Here, Q1 and Q2 are queues with capacities C1 and C2.
//!
//! The splice call for the above diagram would be `rx1.splice(tx2)` or
//! equivalently `tx2.splice(rx1)`.
//! Both forms retain queue 1 and capacity C1 because `rx1` belongs to
//! the upstream channel.
//! The [`Splice`] trait provides this function and is implemented for
//! matching [`Rx`] and [`Tx`] endpoints and for matching [`Duplex`]
//! endpoints.
//!
//! Both channels must be empty, idle, and open when they are spliced.
//! The downstream channel must be local and use the default message
//! sizing and splitting behavior because its configuration is
//! discarded.
//! The upstream channel may have custom configuration because that
//! configuration is retained.
//!
//! [`Attach::attach()`]: crate::Attach::attach()
//! [`Duplex`]: crate::Duplex
//! [`DuplexPump`]: crate::DuplexPump
//! [`Pump`]: crate::Pump
//! [`Rx`]: crate::Rx
//! [`Splice`]: crate::Splice
//! [`Task::block_on()`]: crate::Task::block_on()
//! [`Tx`]: crate::Tx
//! [`Tx::send()`]: crate::Tx::send()

mod activity;
mod activity_lease;
mod activity_monitor;
mod attach;
mod block_on;
mod channel;
mod channel_error;
mod channel_error_info;
mod channel_queue;
mod channel_reservation;
mod channel_state;
mod chunk;
mod duplex;
mod duplex_pump;
mod executor;
mod fan_in_block;
mod fan_out_block;
mod fast_rng;
pub mod latency;
pub mod loss;
mod moment;
mod panicking_writer;
mod poll;
mod poll_set;
mod poll_target;
mod poll_target_ops;
mod pollable;
mod pump;
mod receive_options;
mod rx;
mod send_error;
mod send_options;
mod signal;
mod signal_trigger;
mod spawn;
mod splice;
mod split;
mod task;
mod task_key;
mod task_output;
mod task_queue;
mod task_rng;
mod task_spawner;
mod task_work;
#[cfg(test)]
mod test_error;
mod tx;

pub use activity::Activity;
pub use activity_lease::ActivityLease;
pub use activity_monitor::ActivityMonitor;
pub use attach::Attach;
pub use channel::Channel;
pub use channel::ChannelOrder;
pub use channel_error::ChannelError;
pub use channel_error_info::ChannelErrorInfo;
pub use channel_reservation::ChannelReservation;
pub use chunk::Chunk;
pub use duplex::Duplex;
pub use duplex_pump::DuplexPump;
pub use executor::Executor;
pub use fan_in_block::FanInBlock;
pub use fan_out_block::FanOutBlock;
pub use fast_rng::FastRng;
pub use moment::Moment;
pub use panicking_writer::PanickingWriter;
pub use poll::poll;
pub use poll::poll_with_timeout;
pub use poll_set::PollSet;
pub use poll_set::PollSetEntry;
pub use poll_set::PollSetKey;
pub use poll_target::PollTarget;
pub use pollable::Pollable;
pub use pump::Pump;
pub use receive_options::ReceiveOptions;
pub use rx::Rx;
pub use send_error::SendError;
pub use send_options::SendOptions;
pub use signal::Signal;
pub use signal_trigger::SignalTrigger;
pub use spawn::Spawn;
pub use splice::Splice;
pub use split::Split;
pub use task::Task;
pub use task::TaskId;
pub use task_output::TaskOutput;
pub use task_rng::TaskRng;
pub use tx::Tx;
