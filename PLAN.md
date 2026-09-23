# Statline PLAN

## 1. Logic — hybrid (agreed)

Goal: max sensor coverage, staged, minimal text panel, custom popup charts.

Mechanism from `playbar` (`playbar/src/media.rs:93-119,214-260`):
- Poller runs off UI thread (`std::thread` + `mpsc` -> `Subscription::run` via `stream::channel`).
- `initial_snapshot()` so panel paints instantly, `RECONNECT 1s` backoff.
- Blocking sources (NVML, `lspci`, sysfs, `sysinfo`) never on iced executor.

Orchestration from `syspeek`/`minimon` (`syspeek/src/app/mod.rs:219-250,447-449,734-787`, `syspeek/src/sensors/mod.rs:18-34,108-164`, `syspeek/src/config.rs:25-53`):
- `Sensor { update(), update_config() }` trait, per-sensor files.
- `Tick -> refresh_stats()` with `refresh_rate AtomicU32` (default 3000ms, 0.25..15s spin).
- Visibility gating: popup open polls all, closed polls only `value_visible`.
- `SlowTimer 3s` only for AC/DC GPU `stop()/restart()`.
- History `BoundedVecDeque 21x f64 / 30x u64`, `latest_sample()`, `last_second_rate()`.

Discipline from `vitals` (`vitals/sensors.js:129-184`, `vitals/extension.js:241-255,558-566`, `vitals/helpers/file.js:12-39`, `vitals/values.js:59-238,248-417`):
- Monotonic `dwell` timestamps for all rates, not assumed interval.
- `wantedKeys`-style selective polling (per-sensor, per-hwmon-file when closed).
- Batched `tokio::fs` + cached `PathBuf`s (avoid FD storm), `statvfs` not GTop.
- Central format table + `returnIfDifferent` render-on-change + group Avg/Min/Max + per-sensor alert colors.
- Explicit `disabled` state, log errors (don't swallow).

Sensor stages (one at a time):
1. cpu `/proc/stat` delta
2. memory `sysinfo with_ram`
3. network `sysinfo::Networks`, filter `lo`/virtual
4. disks `sysinfo::Disks with_io_usage`
5. cputemp hwmon `coretemp|k10temp|cpu|zenpower`, `Tdie>ccd>Tctl`
6. fan/voltage generic hwmon scan
7. gpu `GpuIf` + `BTreeMap`, Nvidia `OnceLock<Nvml>`+retry / AMD drm sysfs + hash ID + `lspci` fallback / Intel stub, `pause_on_battery`
8. system: loadavg, uptime, battery `uevent`, fs `statvfs`

License: `playbar` MPL-2.0 may copy; `minimon`/`syspeek` GPL-3.0 re-implement; `vitals` ideas only.

## 2. Styling — agreed

Code rule: every block of code written gets explanatory comments.

Panel (syspeek look, `syspeek/src/app/panel.rs:51-100,384-487`, `syspeek/src/app/mod.rs:256-302`):
- Transparent, no pill/background. Row of sensors in `content_order` order, `panel_spacing` gaps.
- Each sensor: label (semibold) over value (semibold) in `Column` when both visible; fixed measured widths (`8.88%`, `8.88 MB/s`) to avoid jitter; optional monospace + right_align; sizes scale `S+1/M+2/L+3/XL+5`.
- Net/disks: `Column` of 2 `Row`s with `↑/↓` (`U:/D:`) and `R:/W:` markers + values.
- Alert colors on values only (warning orange / critical red). No playbar tint logic.

Popup (layout from `playbar/src/popup/main_page.rs:55-80`, `mod.rs:90-104`, `player_list.rs:145-161`):
- Step 1: main-page top header row only — spacer + info (`help-about-symbolic`) + settings (`preferences-system-symbolic`) icons right, `Link` class tinted by desktop global accent via `Themer`. No left pill yet.
- About page: full now via `widget::about`. Settings page: title `text::title3(settings)` only for now.
- Icons + back button follow desktop global accent, not album color.
- Back buttons and any icon+text button use `custom_row` style: `button::custom(Fill, padding [space_xxs, space_m]).class(row_class(global_accent))`, 14px semibold text + matching icon size, selected = accent fill + contrast text.
