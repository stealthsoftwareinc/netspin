//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

mod executor_cycles_tasks_fairly;
mod multiple_suspensions_in_one_poll_panic;
mod notified_receivers_respect_the_horizon_and_are_dropped;
mod pending_without_a_netspin_suspension_panics;
mod ready_with_a_netspin_suspension_panics;
mod spawn_returns_output;
mod stale_suspension_at_poll_start_panics;
mod stale_wakes_do_not_resume_reused_slots;
mod task_waker_is_reused_across_polls;
mod tasks_only_resume_before_the_horizon;
mod wake_calls_outside_tasks_are_ignored;
mod woken_task_resumes_at_wake_moment;
mod zero_horizon_prevents_tasks_from_starting;
