//
// Copyright 2026 Stealth Software Technologies, Inc.
// SPDX-License-Identifier: Apache-2.0
//

mod abandoned_poll_can_drop_entries;
mod all_empty_waits_until_first_arrival;
mod all_nonempty_waits_until_first_arrival;
mod disabled_entry_is_ignored;
mod disabling_entry_reschedules_unparked_poll;
mod disabling_entry_wakes_parked_poll;
mod empty_set_times_out;
mod enabling_entry_reschedules_unparked_poll;
mod enabling_entry_wakes_parked_poll;
mod entry_enablement_is_idempotent;
mod equal_arrivals_return_first;
mod insertion_reschedules_unparked_poll;
mod insertion_wakes_parked_poll;
mod poll_can_be_called_repeatedly;
mod poller_is_pinned_to_first_task;
mod readiness_wins_at_timeout;
mod ready_is_returned_immediately;
mod reenabled_entries_keep_key_order;
mod removal_interrupts_a_delayed_wakeup_from_park;
mod removal_reschedules_unparked_poll;
mod removal_wakes_parked_poll;
mod removed_entry_stays_alive_until_poll_resumes;
mod signal_can_be_inserted;
