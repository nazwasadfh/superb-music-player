# music

Terminal vinyl-view player for MPRIS-compatible players (Spotify, universal). Real-time cover art, dominant-color accent computation, spinning disc animation, and keyboard control via `playerctl`.

currently only for linux. i am not planning to port it to windows.

---

## 1. Technical / Installation

**Language:** Rust `edition = 2021`  
**Dependencies:** `crossterm`, `image`, `ureq`

**Requirements:**

- `playerctl` working with your player (MPRIS) or Spotify installed.
- A terminal that supports raw mode and alternate screen (`crossterm`).
- `bins/matugen` is included in repo; the app calls it at runtime to extract accent colors from album art.

**Build:**

```bash
cargo build --release
```

**Binaries included (`bins/`):**

- `matugen` — extracts a dominant hex color from an image (used by the TUI for theming).
- `loop-control` / `loop-control-spotify` — helper scripts for locking/unlocking the current track loop.
- `lrc_tty` — lyrics / TUI helper (used for lyric display pipelines).
- `figlet` / `3d.flf` — ASCII art / banner helpers.

**Usage:**

```bash
# Universal MPRIS mode (any supported player)
./target/release/music -u

# Spotify-only mode
./target/release/music -s

# Show help / version
./target/release/music -h
```

**Project files:**

- `src/main.rs` — event loop, rendering, keyboard input, thread management.
- `src/player.rs` — `playerctl` interaction, MPRIS/Spotify send/recv.
- `src/render.rs` — terminal rendering, frame compositing, disc animation.
- `src/art.rs` — async cover art fetching and image processing.
- `screenshots/demo.mp4` — demo recording.

**Debug (single frame, no TTY):**

```bash
MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
```

**Keyboard controls:**

| Key | Action |
| --- | --- |
| `V` | Play / Pause |
| `C` | Lock / Unlock loop for current track |
| `B` | Previous track |
| `N` | Next track |
| `M` | Mute / Restore volume |
| `Z` | Expand / Collapse info panel |
| `D` | Toggle disc texture |
| `Left` / `Right` | Seek backward / forward |
| `Q` | Quit |

**Modes:**

- `Universal` (`-u` / default with args): any MPRIS player via `playerctl`.
- `Spotify` (`-s`): locked to Spotify with `loop-control-spotify` fallback.

---

## 2. What it is / Non-technical

`music` turns your terminal into a minimal record player. It listens to your media player, pulls the current album cover, calculates a color accent from it, and renders a spinning vinyl disc with a needle arm that drops and lifts with playback. The animation is continuous, color-adjusted to your music, and requires zero browser or GUI — just your terminal and `playerctl`.

The design is intentionally restrained: a dark screen, a single disc, and an expanded info view you can toggle with `Z`. It is meant to sit in a split terminal while you work, not to compete with full desktop media apps.

**Demo video (embedded):**

<video src="screenshots/demo.mp4" controls width="100%"></video>

If the embedded player does not render, open `screenshots/demo.mp4` locally.
