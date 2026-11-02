# music

Terminal-vinyl-Player für MPRIS-Plattformen (Spotify, universell). Echtzeit-Cover-Bild, dominanten-Farben-Fokus, sich drehendes Vinyl-Plattendesign und Tastendruck-Steuerung über `playerctl`.

---

## 1. Technisch / Installation

**Sprache:** Rust `edition = 2021`  
**Abhängigkeiten:** `crossterm`, `image`, `ureq`

**Anforderungen:**
- Eine funktionierende `playerctl` (MPRIS) oder Spotify installiert.
- Ein Terminal, das Raw-Modus und die Alternate-Screen-Tabelle unterstützt (`crossterm`).
- `bins/matugen` ist im Repository; der App greift darauf bei der Cover-Bild-Auslesung ab, Dominantenfarben zu berechnen.

**Bauen:**
```bash
cargo build --release
```

**Binare im Repository (`bins/`):**
- `matugen` — extrahiert aus einem Bild einen dominierenden Farbcode (verwendet von der GUI für Thematisierung).
- `loop-control` / `loop-control-spotify` — Skripte zur lokalen Schlechen des aktuellen Titels.
- `lrc_tty` — Titel-Anzeigehilfe.
- `figlet` / `3d.flf` — Bannerskripte / ASCII-Grafiken.

**Bedienung:**
```bash
# Universeller MPRIS-Modus
./target/release/music -u

# Spotify-Modus
./target/release/music -s

# Hilfe
./target/release/music -h
```

**Projekdateien:**
- `src/main.rs` — Zyklus Ereignisbehandlung, Render, Tastatur, Worker-Threads.
- `src/player.rs` — `playerctl`-Interaktion, MPRIS / Spotify.
- `src/render.rs` — Terminal-Render, Frame-Kombination, Vinyl-Animation.
- `src/art.rs` — asynchrones Cover-Bild-Abladen und Grafikbearbeitung.
- `screenshots/demo.mp4` — Demo-Video.

**Debug (ein Frame, ohne TTY):**
```bash
MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
```

**Tastenkombinationen:**
| Taste | Funktion |
|---|---|
| `V` | Play / Pause |
| `C` | Zustellungen festlegen / löschen des aktuellen Songs |
| `B` | Voriges Lied |
| `N` | Nächstes Lied |
| `M` | Stummschalten / Volume Restaurierung |
| `Z` | Info-Panel erweitern / minimieren |
| `D` | Plattentextur wechseln |
| `Links` / `Rechts` | Suchen rückwärts vorwärts |
| `Q` | Beenden |

** Modi:**
- `Universell` (`-u` /Standard mittels Argumetn): jeder MPRIS-Player über `playerctl`.
- `Spotify` (`-s`): Spotify-spezifischer Modus mit `loop-control-spotify`-Fallback.

---

## 2. Was es ist / Deskriptives

`music` wandelt dein Terminal in einen minimalistischen Vinyl-Stuhl um. Es hört auf deinen Medienplayer, holt den aktuellen Albumcover, rechnet den dominanten Farbcode daraus, und rendert eine sich drehende Disc mit einer Nadel, die abhängt und wob samt dem Abspielen. Die Animation ist fließend, farblich angepasst an deine Musik, und braucht nur dein Terminal und `playerctl` — kein Browser und keine GUI.

Das Design ist bewusst reduziert: ein dunkler Bildschirm, eine einzige Disc, und ein erweiterbarer Info-Panel, das du per `Z` aufmachen/schließen kannst. Es passt in einen halbbreiten Terminal während du arbeitest, statt mit vollwertigen Desktop-Medien-Apps zu konkurrieren.

**Demo-Video (eingeblendet):**

<video src="screenshots/demo.mp4" controls width="100%"></video>

Wenn das eingebettete Video nicht läuft, lokal `screenshots/demo.mp4` öffnen.
