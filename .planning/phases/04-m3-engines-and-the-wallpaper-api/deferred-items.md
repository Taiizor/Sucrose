## Deferred Items

- Flaky `apps/sucrose` service test `a_restart_of_another_engine_closes_nothing_and_one_during_a_library_move_is_refused` (assertion `sent_to(&renderers, &logs, slot).is_empty()` at service.rs:6552): failed once in `cargo xtask ci` and once in three isolated runs while 04-32 did not touch service.rs; passed on the next runs. Found during 04-32.
  status: open
