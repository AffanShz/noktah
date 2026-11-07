# Music Dock Apple Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign Noktah’s Music page and dock geometry so navigation reveals downward inside its reserved band, lyrics stay single-line, and transport controls are optically symmetric.

**Architecture:** Keep `layout::content_rect` independent from the dock spring and reserve the full 58px `DOCK_H` band at all times. Introduce a pure geometry helper for the dock’s downward visual reveal, then render its fixed-width tab capsule only inside that band. Build Music controls from equal utility slots around a centered playback trio, and give lyric rows a fixed height with truncation rather than wrapping.

**Tech Stack:** Rust 2021; `egui`/`eframe` 0.29 (`glow`); Windows crate 0.58.

**Spec:** Approved in this session on 2026-11-02; no separate written design document exists.

## Global Constraints

- Keep Noktah attached to the top display edge; do not convert it into a floating Dynamic Island.
- Do not change `OPEN_W = 660.0`, `OPEN_H = 252.0`, or `DOCK_H = 58.0`.
- `content_rect(...).bottom()` must remain `area.bottom() - DOCK_H` for every dock spring value.
- Dock visual expansion must be anchored at the top of `dock_band(area)` and grow downward only within that band.
- The dock tab capsule must retain fixed `DOCK_GROUP_W` width and must never clip the tab row during animation.
- Preserve `NotchPage::SWITCHER` ordering, existing media commands, lyric seek behavior, API-key DPAPI handling, and lack of telemetry.
- Every lyric row must occupy exactly one visual line; overly long text is truncated rather than wrapped.
- Controls must retain their allocated hit rectangles; Apple-style hover/press feedback may alter only painting around the stable center.
- Build the release executable and ZIP into `dist/` after verification; stop a running Noktah executable first if it locks the artifact.

## Review Focus

- **Dock mid-spring (`0 < dock_open < 1`):** capsule, icons, tooltips, and hit rects all remain inside `dock_band(area)`; cover in Task 1 geometry tests.
- **Cursor parked in dock band:** content never moves as dock opens or closes, so hover cannot oscillate; cover in Task 1 content/dock boundary test.
- **Long lyric text:** a line remains one fixed-height row, is visually ellipsized/clipped, and its entire row remains a seek target; cover in Task 3 Egui layout test.
- **Lyrics disabled or panel too narrow:** the central Play control stays centered in the music column and Prev/Next spacing stays mirrored; cover in Task 2 control-row test.
- **Hover and pointer-down on media controls:** paint may scale around center but allocation dimensions and centers stay stable; cover in Task 2 widget/layout test.

---

### Task 1: Downward dock reveal geometry

**Files:**
- Modify: `native/app/src/presentation/layout.rs`
- Modify: `native/app/src/presentation/pages/notch.rs`
- Test: `native/app/src/page_fit_test.rs`

**Interfaces:**
- Consumes: `layout::dock_band(area: Rect) -> Rect`, `DOCK_STRIP_H`, `DOCK_H`, `DOCK_GROUP_W`.
- Produces: `layout::dock_reveal_rect(area: Rect, open: f32) -> Rect`, whose `top()` equals `dock_band(area).top()` and whose height interpolates from `DOCK_STRIP_H` to `DOCK_H`.

- [ ] **Step 1: Write the failing dock-reveal geometry test**

Add a `page_fit_test.rs` test asserting, for `open` values `0.0`, `0.5`, and `1.0`, that:

```rust
let band = layout::dock_band(area);
let reveal = layout::dock_reveal_rect(area, open);
assert_eq!(reveal.top(), band.top());
assert!(band.contains_rect(reveal));
assert!((layout::DOCK_STRIP_H..=layout::DOCK_H).contains(&reveal.height()));
```

Also assert that `layout::content_rect(area, 26.0, style::S3).bottom() == band.top()`.

- [ ] **Step 2: Run the new test to verify it fails**

Run:

```bash
cd native && cargo test -p noktah-native dock_reveal
```

Expected: FAIL because `dock_reveal_rect` is undefined.

- [ ] **Step 3: Implement `pub fn dock_reveal_rect(area: Rect, open: f32) -> Rect`**

In `layout.rs`, clamp `open` to `0.0..=1.0`; calculate `height = DOCK_STRIP_H + (DOCK_H - DOCK_STRIP_H) * open`; return a rect whose top is `dock_band(area).top()` and bottom is `top + height`. Do not modify `content_rect`, `dock_band`, or panel constants.

- [ ] **Step 4: Render the dock from the downward reveal rect**

In `pages/notch.rs`, use `dock_reveal_rect` for strip/capsule placement. Keep the pointer trigger as the full immutable `dock_band(area)`. Render only the strip until the reveal is tall enough for the 34px tab group. At interactive visibility, center the fixed 34px tab row in the reveal and keep capsule width at `DOCK_GROUP_W`; ensure tooltips sit within the dock band rather than above it.

- [ ] **Step 5: Run focused dock tests**

Run:

```bash
cd native && cargo test -p noktah-native dock
```

Expected: PASS, including existing tab-width and content/dock separation tests.

- [ ] **Step 6: Commit the geometry change**

```bash
git add native/app/src/presentation/layout.rs native/app/src/presentation/pages/notch.rs native/app/src/page_fit_test.rs
git commit -m "[AS] fix: reveal dock downward within reserved band"
```

### Task 2: Stable, symmetric music transport controls

**Files:**
- Modify: `native/app/src/presentation/widgets.rs`
- Modify: `native/app/src/presentation/pages/music.rs`
- Test: `native/app/src/layout_test.rs`

**Interfaces:**
- Consumes: `widgets::icon_button`, `widgets::play_button`, `Icon::{ChatBubble, Prev, Play, Pause, Next}`, `style::S2`.
- Produces: an explicit Music transport row with equal utility slots at either side of a centered playback trio; all painted feedback remains centered inside stable allocated rects.

- [ ] **Step 1: Write failing symmetry tests**

Extend `layout_test.rs` with an Egui harness that receives a column width and simulates the new slot layout. Assert:

```rust
assert!((play_center_x - column_center_x).abs() < 0.5);
assert!(((play_center_x - prev_center_x) - (next_center_x - play_center_x)).abs() < 0.5);
assert!((left_utility_center_x - (column_width - right_balance_center_x)).abs() < 0.5);
```

Retain the existing common-Y-center assertion.

- [ ] **Step 2: Run the focused symmetry test to verify it fails**

Run:

```bash
cd native && cargo test -p noktah-native tombol_simetris
```

Expected: FAIL because the old row has no balanced right slot.

- [ ] **Step 3: Implement the balanced Music transport grid**

In `music.rs`, replace the current `IB + pad` placement with three explicit allocations: a left `IB` utility slot containing `ChatBubble`, a center slot containing `Prev`, `Play`, and `Next` at equal `style::S2` gaps, and a right `IB` inert balance slot. Allocate each element inside a fixed `PLAY`-high row using `allocate_ui_with_layout(... Align::Center)` so 36px side controls and 42px Play share the same center.

- [ ] **Step 4: Refine control feedback without changing allocation**

In `widgets.rs`, preserve each button’s allocated rectangle and paint its hover/press scale about `rect.center()`. Request a repaint during pointer-down/hover transitions if existing widget patterns need it. Keep Play visually primary and make Prev/Next use identical scale and icon geometry.

- [ ] **Step 5: Run focused control tests**

Run:

```bash
cd native && cargo test -p noktah-native tombol
```

Expected: PASS for common Y center and new X-axis symmetry assertions.

- [ ] **Step 6: Commit the transport change**

```bash
git add native/app/src/presentation/widgets.rs native/app/src/presentation/pages/music.rs native/app/src/layout_test.rs
git commit -m "[AS] fix: align music transport controls symmetrically"
```

### Task 3: One-line lyric rows and layout regression coverage

**Files:**
- Modify: `native/app/src/presentation/pages/music.rs`
- Modify: `native/app/src/layout_test.rs`
- Test: `native/app/src/page_fit_test.rs`

**Interfaces:**
- Consumes: `self.lyrics`, `Id::new("p_lyric_click").with(idx)`, `noktah_core::lrc::seek_target_ms`, `col_right_w`.
- Produces: a fixed-height lyric row that paints one ellipsized line and retains a full-width click target.

- [ ] **Step 1: Write a failing one-line lyric layout test**

Add an Egui test that lays out a long lyric into a fixed-width lyric column and asserts the allocated row height is a fixed `LYRIC_ROW_H` (for example 20px) rather than a multi-line galley height. Assert the resulting visible galley width is no larger than the column width.

- [ ] **Step 2: Run the new lyric test to verify it fails**

Run:

```bash
cd native && cargo test -p noktah-native lirik_satu_baris
```

Expected: FAIL because the current label uses `TextWrapMode::Wrap`.

- [ ] **Step 3: Implement fixed-height ellipsized lyric rendering**

In `music.rs`, define a local `const LYRIC_ROW_H: f32`. Generate a one-line display string using the existing `widgets::ellipsis` strategy or an equivalent Egui no-wrap galley constrained to `col_right_w`. Allocate exactly `vec2(col_right_w, LYRIC_ROW_H)`, paint the single line vertically centered, and use that row rect for `ui.interact` and active-line auto-scroll. Replace `TextWrapMode::Wrap`; do not change lyric seeking or manual-scroll timing.

- [ ] **Step 4: Extend page-fit coverage**

Keep `verifikasi_semua_halaman_presisi_tanpa_overflow` passing for `NotchPage::Music`, and add a focused assertion that Music’s configured minimum blocks remain below `layout::content_rect(...).bottom()` with the 58px dock reservation.

- [ ] **Step 5: Run lyric and page-fit tests**

Run:

```bash
cd native && cargo test -p noktah-native lirik_satu_baris verifikasi_semua_halaman_presisi_tanpa_overflow
```

Expected: PASS.

- [ ] **Step 6: Commit the lyric layout change**

```bash
git add native/app/src/presentation/pages/music.rs native/app/src/layout_test.rs native/app/src/page_fit_test.rs
git commit -m "[AS] fix: keep synchronized lyrics to one line"
```

### Task 4: Full verification and Windows release package

**Files:**
- Modify: `dist/Noktah-0.3.0-x64.exe`
- Modify: `dist/Noktah-0.3.0-x64.zip`
- Modify: `dist/README.txt` if the package notes enumerate UI changes

**Interfaces:**
- Consumes: passing native workspace tests and the release executable produced by Cargo.
- Produces: fresh release artifacts in `dist/`.

- [ ] **Step 1: Run all native tests**

Run:

```bash
cd native && cargo test --workspace
```

Expected: PASS with no test failures.

- [ ] **Step 2: Build the native release artifact**

Run:

```bash
cd native && cargo build --release -p noktah-native
```

Expected: PASS and produces the Noktah executable in the release target directory.

- [ ] **Step 3: Stop any running Noktah instance and refresh dist artifacts**

Stop the running Noktah process only when necessary to release the destination file lock. Copy the just-built executable to `dist/Noktah-0.3.0-x64.exe` and create `dist/Noktah-0.3.0-x64.zip` containing that executable and release notes.

- [ ] **Step 4: Verify distribution contents**

Run the appropriate Windows file-listing/checksum command and verify the EXE and ZIP are present, non-empty, and dated after the release build.

- [ ] **Step 5: Commit distributable metadata only if tracked**

If this snapshot has version control metadata, commit only tracked text metadata such as `dist/README.txt`; do not commit large generated EXE/ZIP binaries unless the repository already tracks them.
