---
phase: "02"
slug: "m1-video-wallpaper-on-the-desktop-windows"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-11-22"
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. Filled from the `<automated>` commands of plans 02-01 … 02-15 (revision 1, 2026-11-22). Rows for the gap-closure plans 02-16 … 02-22 were added from their own `<automated>` commands on 2026-11-23; their Wave column is the gap-closure run's wave (1–7).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`cargo test`): unit tests next to the code, fixture tests under each crate's `tests/` and `tests/fixtures/`; `#[ignore]` tests for the real libmpv and the real C# install; plus `cargo clippy -D warnings` and `cargo fmt --check` |
| **Config file** | root `Cargo.toml` (`[workspace.lints]`), `rust-toolchain.toml`, `.cargo/config.toml` (`xtask` alias); no extra test config |
| **Quick run command** | the task's own `<automated>` command (usually `cargo test -p <crate> <filter>`) |
| **Full suite command** | `cargo xtask ci` (fmt, clippy `-D warnings`, `test --workspace`), then `cargo xtask fetch-engines && cargo test -p sucrose-engine-mpv -- --ignored` |
| **Estimated runtime** | ~30–180 seconds per task command on an incremental build (runs that start `sucrose.exe` add 8–10 s of waiting); `cargo xtask ci` ~2–4 minutes incremental, longer on a first build (the GPUI portal is in the workspace); plan 15 Task 1 ~7 minutes by design (60 s warm-up + 5 min sampling, D-26) |

All commands run from the repository root in WSL; `cargo` is the Windows toolchain behind a wrapper (Windows binaries). Runs of `sucrose.exe` use `SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)"` with `WSLENV=SUCROSE_DATA_DIR` and are stopped with `taskkill.exe /IM sucrose.exe /F`.

---

## Sampling Rate

- **After every task commit:** run the task's `<automated>` command (map below) and `cargo fmt --all`
- **After every plan wave:** run `cargo xtask ci` (each plan's executor environment requires it before the plan finishes); after plans 01, 07, 11 and 14 also `cargo test -p sucrose-engine-mpv -- --ignored`
- **Before `/gsd-verify-work`:** `cargo xtask ci` and the ignored engine tests must be green
- **Max feedback latency:** ~180 seconds for every task except plan 15 Task 1 (~7 minutes, the D-26 measurement window)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | ENG-01 | T-02-01, T-02-02, T-02-03 | Archive SHA256 checked before extraction, only two named entries extracted to fixed paths; libmpv loaded by absolute path from the exe folder or absolute PATH entries only | unit | `cargo xtask fetch-engines && test -f target/engines/x86_64/libmpv-2.dll && test -f target/engines/x86_64/include/mpv/client.h && test -f target/debug/libmpv-2.dll && cargo test -p xtask` | n/a (run check) or created in the task | ✅ green |
| 02-01-02 | 01 | 1 | ENG-01 | T-02-01, T-02-02, T-02-03 | Archive SHA256 checked before extraction, only two named entries extracted to fixed paths; libmpv loaded by absolute path from the exe folder or absolute PATH entries only | build + lint | `cargo clippy -p sucrose-engine-mpv --all-targets -- -D warnings && cargo clippy -p xtask --all-targets -- -D warnings && cargo fmt --all -- --check` | n/a (run check) or created in the task | ✅ green |
| 02-01-03 | 01 | 1 | ENG-01 | T-02-01, T-02-02, T-02-03 | Archive SHA256 checked before extraction, only two named entries extracted to fixed paths; libmpv loaded by absolute path from the exe folder or absolute PATH entries only | integration (real libmpv or real install) (TDD) | `cargo test -p sucrose-engine-mpv && cargo xtask fetch-engines && cargo test -p sucrose-engine-mpv -- --ignored` | written first in the task | ✅ green |
| 02-02-01 | 02 | 2 | ENG-01 | T-02-04, T-02-05 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking checkpoint, resume signal) | n/a | ✅ resolved (manual) |
| 02-02-02 | 02 | 2 | ENG-01 | T-02-04, T-02-05 | Pin hashes and sizes never change when URLs change; GPL sign-off and source-offer coordinates recorded in D-028 | docs grep | `cargo xtask fetch-engines --arch all && grep -c '^## D-028' docs/DECISIONS.md` | n/a (run check) or created in the task | ✅ green |
| 02-02-03 | 02 | 2 | ENG-01 | T-02-04, T-02-05 | Pin hashes and sizes never change when URLs change; GPL sign-off and source-offer coordinates recorded in D-028 | docs grep | `grep -c 'D-028' .planning/PROJECT.md .planning/REQUIREMENTS.md .planning/ROADMAP.md docs/ARCHITECTURE.md docs/FEATURES.md` | n/a (run check) or created in the task | ✅ green |
| 02-03-01 | 03 | 3 | FMT-08, TYPE-01 | T-02-06 | Legacy Setting files opened read-only; no write API in sucrose-config | e2e run on the real desktop | `cargo build -p sucrose && SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe && grep -h 'selected theme' target/scratch-data/Log/Service.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-04-01 | 04 | 4 | FMT-08 | T-02-07, T-02-08 | Malformed or hand-edited settings fall back to legacy defaults without panics; fixtures carry no profile paths | e2e run on the real desktop | `cargo build -p sucrose && rm -rf target/scratch-data/Log && SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe && grep -h 'legacy settings' target/scratch-data/Log/Service.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-04-02 | 04 | 4 | FMT-08 | T-02-07, T-02-08 | Malformed or hand-edited settings fall back to legacy defaults without panics; fixtures carry no profile paths | unit (TDD) | `cargo test -p sucrose-config && cargo test -p sucrose-core` | written first in the task | ✅ green |
| 02-05-01 | 05 | 5 | TYPE-01, DESK-01 | T-02-09, T-02-10, T-02-11, T-02-12 | Second instance exits; pipe restricted to the current user SID; frame size limits; Hello pid must match the spawned child | e2e run on the real desktop | `cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 4 && SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR timeout 10 target/debug/sucrose.exe; second=$?; taskkill.exe /IM sucrose.exe /F; test "$second" -eq 0 && grep -h 'already running' target/scratch-data/Log/Service.*.log && grep -h 'renderer connected' target/scratch-data/Log/Service.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-05-02 | 05 | 5 | TYPE-01, DESK-01 | T-02-09, T-02-10, T-02-11, T-02-12 | Second instance exits; pipe restricted to the current user SID; frame size limits; Hello pid must match the spawned child | unit (TDD) | `cargo test -p sucrose-ipc && cargo test -p sucrose accept_hello` | written first in the task | ✅ green |
| 02-05-03 | 05 | 5 | TYPE-01, DESK-01 | T-02-09, T-02-10, T-02-11, T-02-12 | Second instance exits; pipe restricted to the current user SID; frame size limits; Hello pid must match the spawned child | e2e run on the real desktop | `cargo test -p sucrose-ipc && cargo test -p sucrose-platform && cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 4 && taskkill.exe /IM sucrose.exe /F && grep -h 'renderer connected' target/scratch-data/Log/Service.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-06-01 | 06 | 6 | DESK-01, DESK-06, TYPE-01 | T-02-13, T-02-34 | Only a relative Source that exists inside the theme folder reaches a renderer; explorer messaging is timeout-bounded | e2e run on the real desktop + human-check | `cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 10 && test "$(tasklist.exe /FI "IMAGENAME eq sucrose.exe" \| grep -c sucrose.exe)" -ge 2; alive=$?; taskkill.exe /IM sucrose.exe /F; test "$alive" -eq 0 && grep -h 'surface attached' target/scratch-data/Log/Renderer-0.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-06-02 | 06 | 6 | DESK-01, DESK-06, TYPE-01 | T-02-13, T-02-34 | Only a relative Source that exists inside the theme folder reaches a renderer; explorer messaging is timeout-bounded | unit (TDD) + human-check | `cargo test -p sucrose-platform` | written first in the task | ✅ green |
| 02-07-01 | 07 | 7 | DESK-01, DESK-06, ENG-01, TYPE-01 | T-02-14, T-02-15 | mpv runs with load-scripts=no, ytdl=no, config=no and input bindings off; a renderer crash stays in its process | e2e run on the real desktop + human-check | `cargo build -p sucrose && cargo xtask fetch-engines && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 10 && test "$(tasklist.exe /FI "IMAGENAME eq sucrose.exe" \| grep -c sucrose.exe)" -ge 2; alive=$?; taskkill.exe /IM sucrose.exe /F; test "$alive" -eq 0 && grep -h 'surface attached' target/scratch-data/Log/Renderer-0.*.log && grep -h 'playback started' target/scratch-data/Log/Renderer-0.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-07-02 | 07 | 7 | DESK-01, DESK-06, ENG-01, TYPE-01 | T-02-14, T-02-15 | mpv runs with load-scripts=no, ytdl=no, config=no and input bindings off; a renderer crash stays in its process | unit (TDD) + human-check | `cargo test -p sucrose-engine-mpv config && cargo test -p sucrose render && cargo clippy -p sucrose-engine-mpv -p sucrose --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-08-01 | 08 | 8 | FMT-02, TYPE-01 | T-02-16, T-02-17, T-02-18 | Canonicalized Source containment; AppVersion gate; rejection texts carry no file contents | integration (real libmpv or real install) | `cargo test -p sucrose-core && cargo test -p sucrose -- --ignored real_install` | n/a (run check) or created in the task | ✅ green |
| 02-08-02 | 08 | 8 | FMT-02, TYPE-01 | T-02-16, T-02-17, T-02-18 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking checkpoint, resume signal) | n/a | ✅ resolved (manual) |
| 02-08-03 | 08 | 8 | FMT-02, TYPE-01 | T-02-16, T-02-17, T-02-18 | Canonicalized Source containment; AppVersion gate; rejection texts carry no file contents | integration (real libmpv or real install) (TDD) | `cargo test -p sucrose-core && cargo test -p sucrose -- --ignored real_install && cargo build -p sucrose` | written first in the task | ✅ green |
| 02-09-01 | 09 | 9 | TRAY-01 | T-02-19, T-02-20, T-02-21 | Tooltip reasons length-limited and only from verified renderers; Quit touches only child handles the service spawned | e2e run on the real desktop | `cargo test -p sucrose-platform icon && cargo test -p sucrose shutdown && cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 8 && taskkill.exe /IM sucrose.exe /F && grep -h 'tray icon ready' target/scratch-data/Log/Service.*.log && cmp apps/sucrose/assets/TrayIcon.ico src/Shared/Sucrose.Shared.Launcher/Assets/TrayIcon.ico` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-09-02 | 09 | 9 | TRAY-01 | T-02-19, T-02-20, T-02-21 | Tooltip reasons length-limited and only from verified renderers; Quit touches only child handles the service spawned | unit + human-check | `cargo test -p sucrose && cargo clippy -p sucrose -p sucrose-ipc -p sucrose-platform --all-targets -- -D warnings` | n/a (run check) or created in the task | ✅ green |
| 02-10-01 | 10 | 10 | DESK-07 | T-02-22, T-02-23 | Bounded restart budget with backoff; job object kills renderers when the service dies | e2e run on the real desktop | `cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 8 && rpid=$(grep -ho 'renderer spawned pid=[0-9]*' target/scratch-data/Log/Service.*.log \| head -n 1 \| grep -o '[0-9]*$') && taskkill.exe /PID "$rpid" /F && sleep 5 && grep -h 'renderer restarted' target/scratch-data/Log/Service.*.log && spid=$(grep -ho 'service started pid=[0-9]*' target/scratch-data/Log/Service.*.log \| head -n 1 \| grep -o '[0-9]*$') && taskkill.exe /PID "$spid" /F && sleep 3 && test "$(tasklist.exe /FI "IMAGENAME eq sucrose.exe" \| grep -c sucrose.exe)" -eq 0` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-10-02 | 10 | 10 | DESK-07 | T-02-22, T-02-23 | Bounded restart budget with backoff; job object kills renderers when the service dies | unit (TDD) | `cargo test -p sucrose supervisor` | written first in the task | ✅ green |
| 02-11-01 | 11 | 11 | ENG-01 | T-02-24, T-02-25 | mpv options only from typed values and an allow-list; user uMpvPlayer.config applied after the embedding options | e2e run on the real desktop | `cargo test -p sucrose-engine-mpv config && cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 12 && taskkill.exe /IM sucrose.exe /F && grep -h 'playback started' target/scratch-data/Log/Renderer-0.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-11-02 | 11 | 11 | ENG-01 | T-02-24, T-02-25 | mpv options only from typed values and an allow-list; user uMpvPlayer.config applied after the embedding options | integration (real libmpv or real install) (TDD) | `cargo test -p sucrose-ipc && cargo xtask fetch-engines && cargo test -p sucrose-engine-mpv -- --ignored` | written first in the task | ✅ green |
| 02-12-01 | 12 | 12 | DESK-01, DESK-06, DESK-07 | T-02-26, T-02-27 | Display and shell events debounced; one renderer per monitor id | e2e run on the real desktop | `cargo test -p sucrose topology && cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 10 && taskkill.exe /IM sucrose.exe /F && grep -h 'renderers started' target/scratch-data/Log/Service.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-12-02 | 12 | 12 | DESK-01, DESK-06, DESK-07 | T-02-26, T-02-27 | Display and shell events debounced; one renderer per monitor id | unit (TDD) + human-check | `cargo test -p sucrose topology && cargo test -p sucrose debounce && cargo clippy --workspace --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-12-03 | 12 | 12 | DESK-01, DESK-06, DESK-07 | T-02-26, T-02-27 | Display and shell events debounced; one renderer per monitor id | unit (TDD) + human-check | `cargo test -p sucrose supervisor && cargo test -p sucrose shell_restart && cargo build -p sucrose` | written first in the task | ✅ green |
| 02-13-01 | 13 | 13 | PERF-05 | T-02-28, T-02-29 | Foreground probe never reads window titles; 5/3 debounce limits flips | unit | `cargo test -p sucrose fullscreen && cargo test -p sucrose-platform && cargo build -p sucrose` | n/a (run check) or created in the task | ✅ green |
| 02-13-02 | 13 | 13 | PERF-05 | T-02-28, T-02-29 | Foreground probe never reads window titles; 5/3 debounce limits flips | unit (TDD) + human-check | `cargo test -p sucrose fullscreen && cargo clippy -p sucrose --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-14-01 | 14 | 14 | DESK-01 | T-02-30, T-02-31 | Only the current master's Clock is relayed; seeks only above 0.5 s drift | e2e run on the real desktop | `cargo test -p sucrose-engine-mpv sync && cargo build -p sucrose && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/debug/sucrose.exe &) && sleep 10 && taskkill.exe /IM sucrose.exe /F && grep -h 'role' target/scratch-data/Log/Renderer-0.*.log` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-14-02 | 14 | 14 | DESK-01 | T-02-30, T-02-31 | Only the current master's Clock is relayed; seeks only above 0.5 s drift | unit (TDD) + human-check | `cargo test -p sucrose-engine-mpv sync && cargo test -p sucrose master` | written first in the task | ✅ green |
| 02-15-01 | 15 | 15 | MEAS-01 | T-02-32, T-02-33 | Measurement only reads process statistics; never stops or signals C# processes | e2e run on the real desktop | `cargo test -p xtask measure && cargo build --release -p sucrose && cargo xtask fetch-engines && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/release/sucrose.exe &) && sleep 2 && cargo xtask measure --side rust --out target/measure/rust.json; code=$?; taskkill.exe /IM sucrose.exe /F; test "$code" -eq 0 && test -s target/measure/rust.json` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-15-02 | 15 | 15 | MEAS-01 | T-02-32, T-02-33 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking checkpoint, resume signal) | n/a | ✅ resolved (manual) |
| 02-15-03 | 15 | 15 | MEAS-01 | T-02-32, T-02-33 | Measurement only reads process statistics; never stops or signals C# processes | docs grep | `test -s target/measure/csharp.json && grep -c 'M1 video wallpaper' docs/MEASUREMENTS.md` | n/a (run check) or created in the task | ✅ green |
| 02-16-01 | 16 | 1 | DESK-07, DESK-01 | T-02-35, T-02-36, T-02-37 | WM_DRAIN is posted at most once per pending batch; the order guard is limited to Progman's process and events 0x8000..0x8004, restores at most 10 times per 60 s and only reorders the renderer's own surface | unit (TDD) | `cargo test -p sucrose-platform sent_ && cargo clippy -p sucrose-platform --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-16-02 | 16 | 1 | DESK-07, DESK-01 | T-02-35, T-02-36, T-02-37 | WM_DRAIN is posted at most once per pending batch; the order guard is limited to Progman's process and events 0x8000..0x8004, restores at most 10 times per 60 s and only reorders the renderer's own surface | unit (TDD) | `cargo test -p sucrose-platform --test layout raised_order && cargo test -p sucrose-platform && cargo clippy -p sucrose-platform --all-targets -- -D warnings` | written first in the task | ✅ green (the WorkerW wait itself is manual-only, see below) |
| 02-16-03 | 16 | 1 | DESK-07, DESK-01 | T-02-35, T-02-36, T-02-37 | WM_DRAIN is posted at most once per pending batch; the order guard is limited to Progman's process and events 0x8000..0x8004, restores at most 10 times per 60 s and only reorders the renderer's own surface | e2e run on the real desktop + human-check | `cargo test -p sucrose-platform && cargo clippy -p sucrose-platform --all-targets -- -D warnings && cargo build --release -p sucrose && cargo xtask fetch-engines && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/release/sucrose.exe &) && sleep 12; taskkill.exe /IM sucrose.exe /F; grep -h 'surface attached' target/scratch-data/Log/Renderer-0.*.log \| grep -c 'SHELLDLL_DefView > SucroseDesktopSurface'` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence; the order guard budget is manual-only, see below) |
| 02-17-01 | 17 | 2 | DESK-12 | T-02-38 | Only the value named by Sucrose's own exe path under HKCU is read and written; other pairs kept, nothing deleted, an unchanged value not rewritten; tests use a scratch subkey, never the real DirectX key | unit (TDD) | `cargo test -p sucrose-core settings && cargo test -p sucrose-platform gpu && cargo clippy -p sucrose-core -p sucrose-platform --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-17-02 | 17 | 2 | DESK-12 | T-02-38 | Only the value named by Sucrose's own exe path under HKCU is read and written; other pairs kept, nothing deleted, an unchanged value not rewritten; tests use a scratch subkey, never the real DirectX key | unit (TDD) | `cargo test -p sucrose-platform gpu && cargo clippy -p sucrose-platform --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-18-01 | 18 | 3 | DESK-12, ENG-01 | T-02-39, T-02-40 | GraphicPreference accepts only the three names or ordinals 0..2, anything else gives HighPerformance; logs carry the preference, the outcome and the adapter name/LUID, never the exe path | unit (TDD) + e2e run on the real desktop | `cargo test -p sucrose-config graphic_preference && cargo test -p sucrose && cargo build --release -p sucrose && cargo xtask fetch-engines && …` (release run in the scratch data dir: the GPU Engine counters of the sucrose.exe pids name the LUID of the "video GPU adapter" log line, which is the NVIDIA adapter, and `reg.exe query` shows the UserGpuPreferences value; full command in the 02-18-PLAN Task 1 `<automated>` block) | written first in the task | ✅ green (unit; run: SUMMARY evidence) + gap test below |
| 02-18-02 | 18 | 3 | DESK-12 | T-02-39, T-02-40 | GraphicPreference accepts only the three names or ordinals 0..2, anything else gives HighPerformance; logs carry the preference, the outcome and the adapter name/LUID, never the exe path | docs grep | `grep -c 'pulled into Phase 2 by the UAT' docs/FEATURES.md && grep -c '\| DESK-12 \| Phase 2 \| Complete \|' .planning/REQUIREMENTS.md && grep -c 'UserGpuPreferences' docs/ARCHITECTURE.md` | n/a (run check) or created in the task | ✅ green |
| 02-19-01 | 19 | 4 | DESK-07, DESK-01 | T-02-41, T-02-42 | The slot of a Playing message comes from the connection verified at Hello, never from message content; elections run only on Playing, exit and applied topology changes, and the sticky master prevents role ping-pong | unit (TDD) + e2e run on the real desktop | `cargo test -p sucrose master && cargo test -p sucrose-engine-mpv && cargo build --release -p sucrose && cargo xtask fetch-engines && rm -rf target/scratch-data/Log && (SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)" WSLENV=SUCROSE_DATA_DIR target/release/sucrose.exe &) && sleep 12; taskkill.exe /IM sucrose.exe /F; awk '/playback started/ && !p {p=NR} /role=Master/ && !m {m=NR} END {exit !(p && m && p < m)}' target/scratch-data/Log/Renderer-0.*.log` | written first in the task | ✅ green (unit; run: SUMMARY evidence) |
| 02-19-02 | 19 | 4 | DESK-07, DESK-01 | T-02-41, T-02-42 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking decision checkpoint; the maintainer chose the sticky master, D-030) | n/a | ✅ resolved (manual) |
| 02-19-03 | 19 | 4 | DESK-07, DESK-01 | T-02-41, T-02-42 | The slot of a Playing message comes from the connection verified at Hello, never from message content; elections run only on Playing, exit and applied topology changes, and the sticky master prevents role ping-pong | unit (TDD) + docs grep | `cargo test -p sucrose master && cargo test -p sucrose-ipc && cargo clippy -p sucrose --all-targets -- -D warnings && grep -c 'Master clock election' docs/DECISIONS.md` | written first in the task | ✅ green |
| 02-20-01 | 20 | 5 | DESK-01 | T-02-43 | Follower speed stays within ±5 % of the master's, changes below 0.1 % are suppressed, seeks only beyond 0.5 s and at most one per 2 s, the integral is clamped (anti-windup) | unit (TDD) | `cargo test -p sucrose-engine-mpv sync && cargo clippy -p sucrose-engine-mpv --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-20-02 | 20 | 5 | DESK-01 | T-02-43 | Follower speed stays within ±5 % of the master's, changes below 0.1 % are suppressed, seeks only beyond 0.5 s and at most one per 2 s, the integral is clamped (anti-windup) | unit + docs grep | `cargo test -p sucrose && cargo clippy -p sucrose --all-targets -- -D warnings && grep -c 'follower drift summary' apps/sucrose/src/render.rs && grep -c 'Follower correction (02-20)' docs/FEATURES.md` | n/a (run check) or created in the task | ✅ green + gap test below |
| 02-21-01 | 21 | 6 | MEAS-01 | T-02-44, T-02-45 | The measure task only reads process statistics and counters and never starts, stops or changes a C# process, dwm.exe or a setting; a cross-adapter pair gives no row; adapter names and LUIDs only (accepted) | unit (TDD) | `cargo test -p xtask measure && cargo clippy -p xtask --all-targets -- -D warnings` | written first in the task | ✅ green |
| 02-21-02 | 21 | 6 | MEAS-01 | T-02-44, T-02-45 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking checkpoint, resume signal) | n/a | ✅ resolved (manual) |
| 02-21-03 | 21 | 6 | MEAS-01 | T-02-44, T-02-45 | The measure task only reads process statistics and counters and never starts, stops or changes a C# process, dwm.exe or a setting; a cross-adapter pair gives no row; adapter names and LUIDs only (accepted) | e2e measurement on the real desktop | `test -s target/measure/csharp-same-gpu.json && grep -c 'gpu_adapter' target/measure/csharp-same-gpu.json` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-22-01 | 22 | 7 | MEAS-01, DESK-12 | T-02-46, T-02-47 | Maintainer decision or action; no code | checkpoint (maintainer) | — (blocking checkpoint, resume signal) | n/a | ✅ resolved (manual) |
| 02-22-02 | 22 | 7 | MEAS-01, DESK-12 | T-02-46, T-02-47 | Only deband and the profile line may change and the hardening options stay (re-checked by tests/options.rs); the replaced cross-GPU rows are named in a "compared two GPUs" note | e2e measurement on the real desktop + docs grep | `test -s target/measure/rust-same-gpu.json && grep -c 'gpu_adapter' target/measure/rust-same-gpu.json && grep -c 'compared two GPUs' docs/MEASUREMENTS.md && grep -c 'same GPU' docs/MEASUREMENTS.md` | n/a (run check) or created in the task | ✅ green (SUMMARY evidence) |
| 02-22-03 | 22 | 7 | MEAS-01, DESK-12 | T-02-46, T-02-47 | Only deband and the profile line may change and the hardening options stay (re-checked by tests/options.rs); the replaced cross-GPU rows are named in a "compared two GPUs" note | conditional (only if R > 1.25) | `cargo test -p sucrose-engine-mpv && cargo xtask ci` | n/a | ✅ green (condition not met: R = 1.02, no change made) |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

### Gap-closing tests (validate-phase, 2026-11-22)

| Task ID | Requirement | Behavior | Automated Command | Status |
|---------|-------------|----------|-------------------|--------|
| 02-11-01 / 02-07-02 | ENG-01 (T-02-14, T-02-25) | A user `uMpvPlayer.config` cannot turn `load-scripts`, `ytdl` or `input-default-bindings` back on; a user `volume` still applies; a config that cannot load falls back to the generated options | `cargo test -p sucrose-engine-mpv --test options -- --ignored user_config` | ✅ green |
| 02-06-02 | DESK-06 | `init_dpi_awareness()` reports Per-Monitor V2 in a fresh process | `cargo test -p sucrose-platform --test dpi` | ✅ green |
| 02-13-01 | PERF-05 (D-008) | `Fleet::enable_fullscreen` stays off for Resume (no 1 s timer) and turns on for Pause and Close | `cargo test -p sucrose enable_fullscreen` | ✅ green |

### Gap-closing tests (validate-phase, 2026-11-23)

| Task ID | Requirement | Behavior | Automated Command | Status |
|---------|-------------|----------|-------------------|--------|
| 02-18-01 | DESK-12 (T-02-40) | `apply_gpu_preference` writes nothing when `Capabilities.gpu_preference` is false, writes once for this executable with the given preference when it is true, and returns normally (one attempt, no retry) when the write fails; the fake `Platform` never touches the registry | `cargo test -p sucrose gpu_preference` | ✅ green |
| 02-20-02 | DESK-01 | For a playing 30 fps lavfi stream the pinned libmpv reports `container-fps` (else `estimated-vf-fps`), read the way `render.rs` reads it, and `sync::frame_period` gives 1/30 s | `cargo test -p sucrose-engine-mpv --test options -- --ignored frame_rate` | ✅ green |

Map correction (validate-phase, 2026-11-23, after the code review fixes): the 02-19 sticky election renamed the `elect_master` tests, so `cargo test -p sucrose elect_master` matched no test and passed empty; rows 02-14-02 and "Same frame and no echo across monitors" now use `cargo test -p sucrose master` (12 `service::tests` election tests). `cargo test --workspace` green (350 passed, 12 ignored).

Escalated to manual-only (no implementation change): 02-16-02 / 02-16-03, DESK-01 (T-02-36): the raised attach waiting for the wallpaper WorkerW and the order guard restore budget are bound to the live Progman tree, `SetWinEventHook` and a thread-local budget with no pure seam; see Manual-Only Verifications.

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements: the Rust test harness needs no install, and every task writes its own tests (TDD tasks write them first). Prerequisites that are not test files:

- [x] `cargo xtask fetch-engines` (plan 01 Task 1) before any `-- --ignored` engine test or any run that plays video
- [x] Docker Desktop running, or LLVM for Windows with `LIBCLANG_PATH`, for the one-time bindgen run (plan 01 Task 2 precondition)
- [x] The C# Sucrose app exited from its tray before runs that start `sucrose.exe` (precondition of plans 06, 07, 09, 12; D-07)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Surface and video visible behind the icons; icons clickable; no taskbar or Alt-Tab entry | DESK-01, TYPE-01 | DWM composition on the real 26300 desktop is only observable by eye | Plan 06 Task 1 and plan 07 Task 1 human-checks (strategy swap if needed, recorded in the SUMMARYs) |
| Sharp at a non-100 % display scale | DESK-06 | Sharpness and scale changes need the maintainer's display settings | Plan 06 Task 2 (rect) and plan 07 Task 2 (video) human-checks; plan 12 Task 2 (scale change while running) |
| Tray icon, tooltip reasons, Quit | TRAY-01 | Clicking the tray menu cannot be automated (CLAUDE.md: never synthesize input) | Plan 09 Task 2 human-check |
| explorer.exe restart recovery | DESK-07 | Kills and restarts the maintainer's shell | Plan 12 Task 3 human-check |
| Monitor plug/unplug and "every monitor" beyond one | DESK-01, DESK-07 | The maintainer's machine has one display; needs a second or virtual display | Plan 12 Task 2 human-check if available, else "not verifiable: one display"; the logic is covered by `cargo test -p sucrose topology`, `debounce` and `shell_restart`, and the 02-12 and 02-15 SUMMARYs state that success criteria 1 and 2 were verified for one monitor only |
| Same frame and no echo across monitors | DESK-01 (D-02) | Needs two displays | Plan 14 Task 2 human-check if available; logic covered by `cargo test -p sucrose-engine-mpv sync` and `cargo test -p sucrose master` |
| Fullscreen Pause (and Close) | PERF-05 | Needs a real fullscreen app and a maintainer-made, restored edit of Backgroundog.json (D-08) | Plan 13 Task 2 human-check |
| C# side of the measurement | MEAS-01 | The maintainer starts the C# app; Sucrose never starts or stops it | Plan 15 Task 2 checkpoint, then Task 3 |
| Service pipe refuses a second Windows account | TYPE-01 (T-02-11) | Needs a second Windows account on the machine | Start the service as one user, connect a client from another account; expect access denied (02-05 SUMMARY) |
| libmpv pin source and GPL sign-off; version form | ENG-01, FMT-02 | Maintainer decisions (the version form is one-way) | Plan 02 Task 1 and plan 08 Task 2 checkpoints |
| Raised attach waits for the shell's wallpaper WorkerW (0x052C resent until the 10 s deadline); the WinEvent order guard puts the surface back above a WorkerW the shell creates or reorders later, at most 10 restores per 60 s, then unhooks | DESK-01 (T-02-36) | Bound to the live Progman/WorkerW tree, `SetWinEventHook` and a thread-local budget with no pure seam (validate-phase 2026-11-23 escalated it without changing the implementation); the order rule itself is covered by `cargo test -p sucrose-platform --test layout raised_order` | UAT test 2: restart explorer.exe twice in a row; the video comes back on every monitor, the Service log shows the restart within a few seconds and the Renderer log shows "surface attached" with `SHELLDLL_DefView > SucroseDesktopSurface > WorkerW` or an "order restored" line (02-16 SUMMARY, harvested check) |
| Non-primary monitor unplug and re-plug applied within about 3 s with FullScreenPerformance at Resume | DESK-07 (G-02-2d) | Needs a second or virtual display on the maintainer's desktop | With FullScreenPerformance = Resume, disable and re-enable the virtual display; only that monitor's renderer closes and comes back within about 3 s (02-16 SUMMARY, harvested check); the sent-message delivery is covered by `cargo test -p sucrose-platform sent_` |
| Renderers run on the high-performance GPU and 4K60 plays smoothly | DESK-12, ENG-01 | Task Manager and smoothness need the maintainer's eyes | Task Manager "GPU engine" column shows the NVIDIA GPU for the renderers, and 2 minutes of 4K60 playback stay smooth (02-18 SUMMARY, harvested check); machine evidence (counter LUID = "video GPU adapter" line, registry value) is in the 02-18 SUMMARY; the service logic is covered by `cargo test -p sucrose gpu_preference` |
| Sticky master across a monitor change | DESK-01, DESK-07 (D-030) | Needs a virtual display | Disable and re-enable the virtual display; the monitor that kept playing does not seek back to the start and the Service log shows no master move (02-19 SUMMARY, harvested check); logic covered by `cargo test -p sucrose master` |
| Followers converge to the master's frame | DESK-01 (G-02-2c) | Needs two displays and the maintainer's eyes | Plug a virtual display, compare both monitors for the same frame and read the "frame period" and "follower drift summary" lines in the Renderer logs (02-20 SUMMARY, harvested check); the controller is covered by `cargo test -p sucrose-engine-mpv sync` and the frame period source by `cargo test -p sucrose-engine-mpv --test options -- --ignored frame_rate` |
| Same-GPU measurement of both sides | MEAS-01 | The maintainer starts and exits the C# app; Sucrose never starts or stops it | Plan 21 Task 2 and plan 22 Task 1 checkpoints, then 02-21 Task 3 and 02-22 Task 2 (`cargo xtask measure`, which refuses a cross-adapter row) |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (the three checkpoint tasks are maintainer decisions or actions with resume signals)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none: no task uses a MISSING placeholder)
- [x] No watch-mode flags
- [x] Feedback latency < 180s (holds for every task except plan 15 Task 1, whose 6-minute measurement window is part of D-26 — exception accepted)
- [x] `nyquist_compliant: true` set in frontmatter (validate-phase accepted the plan 15 exception and the manual-only rows above)

**Approval:** validated 2026-11-22 (`/gsd-validate-phase 02`: 3 gaps found, 3 resolved, 0 escalated)

**Approval (gap closure):** validated 2026-11-23 (`/gsd-validate-phase 02` for plans 02-16 … 02-22: 3 gaps found, 2 resolved, 1 escalated to manual-only). Every new task has an automated command or is a maintainer checkpoint; the plan 15 latency exception also covers the measurement windows behind 02-21 Task 3 and 02-22 Task 2 (60 s warm-up + 300 s).

## Validation Audit 2026-11-22

| Metric | Count |
|---|---|
| Gaps found | 3 |
| Resolved | 3 |
| Escalated | 0 |

## Validation Audit 2026-11-23

| Metric | Count |
|---|---|
| Gaps found | 3 |
| Resolved | 2 |
| Escalated | 1 |

## Validation Audit 2026-11-23

| Metric | Count |
|---|---|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |
