use std::process::Command;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Spotify,
    Universal,
}

const SEP: &str = "...A..."; // separator, który nie pojawi się w tytułach
const MD_FMT: &str = "{{title}}...A...{{artist}}...A...{{album}}...A...{{mpris:artUrl}}...A...{{mpris:length}}...A...{{status}}...A...{{playerName}}";

#[derive(Default, Clone)]
pub struct Info {
    pub present: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: Option<String>,
    pub length_sec: f64,
    pub status: String,
    pub player: String,
    pub position: f64,
    pub volume: f64,
    pub loop_status: String,
    /// Zeitpunkt, w którym main przyjął ten snapshot (do interpolacji pozycji).
    pub at: Option<Instant>,
    pub accent: String,
}

fn ctl(mode: Mode, player: &str, rest: &[&str]) -> Option<String> {
    let mut cmd = Command::new("playerctl");
    match mode {
        Mode::Spotify => {
            cmd.args(["-p", "spotify"]);
        }
        Mode::Universal => {
            if !player.is_empty() {
                cmd.args(["-p", player]);
            }
        }
    }
    cmd.args(rest);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn first_line(s: &str) -> &str {
    s.lines().find(|l| !l.trim().is_empty()).unwrap_or("")
}

fn parse_volume(raw: &str) -> Option<f64> {
    let v = raw.trim().parse::<f64>().ok()?;
    let v = if v > 1.0 { v / 100.0 } else { v };
    Some(v.clamp(0.0, 1.0))
}

fn poll_info(mode: Mode) -> Info {
    let md = match ctl(mode, "", &["metadata", "-f", MD_FMT]) {
        Some(m) => m,
        None => return Info::default(),
    };

    // universal: playerctl może wypisać wiele linii (po jednej na player) —
    // wybieramy tę, gdzie status == Playing, inaczej pierwszą niepustą.
    let mut best: Option<&str> = None;
    for line in md.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains("...A...Playing") {
            best = Some(line);
            break;
        }
        if best.is_none() {
            best = Some(line);
        }
    }
    let line = match best {
        Some(l) => l,
        None => return Info::default(),
    };

    let parts: Vec<&str> = line.split(SEP).collect();
    if parts.len() < 7 {
        return Info::default();
    }

    let art_url = parts[3].trim();
    let length_sec = parts[4]
        .trim()
        .parse::<f64>()
        .ok()
        .map(|us| us / 1_000_000.0)
        .unwrap_or(0.0);
    let status = parts[5].trim().to_string();
    let player_name = parts[6].trim().to_string();

    let player = if player_name.is_empty() {
        match mode {
            Mode::Spotify => "spotify".to_string(),
            Mode::Universal => String::new(),
        }
    } else {
        player_name
    };

    let position = ctl(mode, &player, &["position"])
        .map(|s| first_line(&s).trim().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);
    let volume = ctl(mode, &player, &["volume"])
        .and_then(|s| parse_volume(first_line(&s)))
        .unwrap_or(0.0);
    let loop_status = ctl(mode, &player, &["loop"])
        .map(|s| first_line(&s).trim().to_string())
        .unwrap_or_default();

    Info {
        present: true,
        title: parts[0].trim().to_string(),
        artist: parts[1].trim().to_string(),
        album: parts[2].trim().to_string(),
        art_url: if art_url.is_empty() { None } else { Some(art_url.to_string()) },
        length_sec,
        status,
        player,
        position,
        volume,
        loop_status,
        at: None,
        accent: "".into(),
    }
}

pub fn info_worker(mode: Mode, tx: SyncSender<Info>) {
    loop {
        let info = poll_info(mode);
        let _ = tx.try_send(info);
        thread::sleep(Duration::from_millis(280));
    }
}

// --- akcje z keybindów -----------------------------------------------

pub fn send_action(mode: Mode, player: &str, action: &str) {
    ctl(mode, player, &[action]);
}

/// Przewijanie o sekundy (positive = forward, negative = backward).
pub fn seek_rel(mode: Mode, player: &str, sec: f64) {
    if let Some(pos) = ctl(mode, player, &["position"]) {
        if let Ok(cur) = pos.trim().parse::<f64>() {
            let new = (cur + sec).max(0.0);
            let _ = ctl(mode, player, &["position", &format!("{}", new)]);
        }
    }
}

pub fn volume_set(mode: Mode, player: &str, v: f64) {
    ctl(mode, player, &["volume", &format!("{}", v)]);
}

pub fn loop_lock(mode: Mode, player: &str, lock: bool) {
    ctl(mode, player, &["loop", if lock { "Track" } else { "None" }]);
}

/// Uruchamia skrypt skrypt = "loop-control" / "loop-control-spotify".
/// Szuka go obok binarki, dwa poziomy wyżej (root projektu) i w cwd.
pub fn run_loop_script(script: &str) -> bool {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            // skrypty mogą być w bins/ obok binarki
            candidates.push(d.join("bins").join(script));
            candidates.push(d.join(script));
            if let Some(p) = d.parent().and_then(|p| p.parent()) {
                candidates.push(p.join("bins").join(script));
                candidates.push(p.join(script));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("bins").join(script));
        candidates.push(cwd.join(script));
    }
    for p in candidates {
        if p.exists() {
            return Command::new(&p)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok();
        }
    }
    false
}
