//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

use core::task::Waker;

use crate::moment::Moment;
use crate::task::Task;
use crate::task_key::TaskKey;

fn task(key: TaskKey) -> Task {
  Task {
    future: Box::pin(async {}),
    key,
    moment: Moment::ZERO,
    waker: Waker::noop().clone(),
  }
}

mod current_moment_tasks_keep_push_order;
mod mixed_operations_match_reference_order;
mod notifications_before_a_wake_keep_earliest_arrival;
mod parked_notifications_preserve_wake_order;
#[cfg(debug_assertions)]
mod pushing_before_current_moment_panics;
mod rebuilding_preserves_parked_tasks;
mod rebuilding_preserves_pending_notifications;
mod rebuilding_the_priority_queue_preserves_current_tasks;
mod rescheduling_a_current_task_is_a_noop;
#[cfg(debug_assertions)]
mod rescheduling_before_current_moment_panics;
mod rescheduling_never_delays_a_task;
mod rescheduling_preserves_original_push_order;
mod rescheduling_rebuilds_a_stale_queue;
mod rescheduling_to_current_moment_preserves_push_order;
mod slot_storage_is_bounded_by_live_tasks;
mod stale_heap_entries_preserve_reused_slot_order;
mod stale_queue_entries_do_not_repeat_tasks;
mod stale_wakes_do_not_wake_reused_slots;
mod tasks_pop_in_moment_then_push_order;
mod wakes_only_schedule_parked_tasks_once;
