mod art;
mod player;
mod render;

use player::{Mode, Info};
use render::Screen;

use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};

const HELP: &str = "\
music 0.1.0 — spinning vinyl terminal view for playerctl

USAGE:
  music -s, --spotify      Spotify only (lock-next: loop-control-spotify)
  music -u, --universal    any MPRIS player (lock-next: loop-control)
  music                    this help

KEYS:
  [V] play-pause           pause / resume
  [C] lock-current         loop (or unlock) current track
  [B] prev-track           previous track
  [N] next-track           next track
  [M] mute                 mute / restore volume
  [Z] show keybinds        show / hide shortcuts
  [Q] quit                 exit
";

fn print_help() {
    print!("{}", HELP);
}

fn parse_mode() -> Result<Mode, ()> {
    let mut mode: Option<Mode> = None;
    let mut any_arg = false;
    for a in std::env::args().skip(1) {
        any_arg = true;
        match a.as_str() {
            "-s" | "--spotify" => {
                if mode.is_some() {
                    eprintln!("provide only one mode: -s or -u");
                    Err(())?;
                }
                mode = Some(Mode::Spotify);
            }
            "-u" | "--universal" => {
                if mode.is_some() {
                    eprintln!("provide only one mode: -s or -u");
                    Err(())?;
                }
                mode = Some(Mode::Universal);
            }
            "-h" | "--help" | "--version" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("unknown argument: {}", a);
                Err(())?;
            }
        }
    }
    match mode {
        Some(m) => Ok(m),
        None => {
            if !any_arg {
                // `music` without args behaves like --help
                print_help();
                std::process::exit(0);
            } else {
                Err(())
            }
        }
    }
}

/// Tryb debug: MUSIC_FRAME_OUT=/tmp/frame.ans — rysuje jedną klatkę
/// (z symulowanym stanem), zapisuje do pliku i wychodzi. Bez PTY.
fn debug_frame(path: &str) {
    let mut info = Info {
        present: true,
        title: "Not Afraid".to_string(),
        artist: "Eminem".to_string(),
        album: "Recovery".to_string(),
        art_url: Some("https://fake/i.jpg".to_string()),
        length_sec: 246.0,
        status: "Playing".to_string(),
        player: "spotify".to_string(),
        position: 75.0,
        volume: 0.85,
        loop_status: "Track".to_string(),
        at: None,
        accent: "".into(),
    };
    // jeśli artefakt testowy istnieje obok, użyj go (metryka faktycznego obrazu)
    let mut loaded: Option<(String, art::Img)> = None;
    for cand in ["test-art.png", "test-art.jpg", "art.png", "cover.jpg"] {
        if std::path::Path::new(cand).exists() {
            if let Some(img) = image::open(cand).ok() {
                loaded = Some((info.art_url.clone().unwrap(), img.to_rgb8()));
                break;
            }
        }
    }

    // symulowane stany dla debugowania: MUSIC_PAUSED=1, MUSIC_ARM_T=0..1
    let paused = std::env::var("MUSIC_PAUSED").ok().as_deref() == Some("1");
    if paused {
        info.status = "Paused".to_string();
    }
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let mut screen = render::Screen::new(tw as usize, th as usize);
    let view = render::View {
        info: &info,
        expanded: false,
        muted: false,
        disp_pos: info.position,
        spin: 1.0,
        arm_t: std::env::var("MUSIC_ARM_T")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(if paused { 0.0 } else { 1.0 }),
        disc_texture: true,
    };
    render::draw(&mut screen, &view, loaded.as_ref());
    let _ = std::fs::write(path, screen.compose());
}

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Self {
        let mut out = std::io::stdout();
        let _ = terminal::enable_raw_mode();
        let _ = execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All));
        Self
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = execute!(out, Show, LeaveAlternateScreen, Clear(ClearType::FromCursorDown));
        let _ = terminal::disable_raw_mode();
    }
}

fn lock_current(mode: Mode, info: &Info) {
    // najpierw spróbuj dedykowanego skryptu (z notify-send itd.)
    let script = if mode == Mode::Spotify {
        "loop-control-spotify"
    } else {
        "loop-control"
    };
    if !player::run_loop_script(script) {
        // fallback: samo playerctl
        let lock = info.loop_status != "Track";
        player::loop_lock(mode, &info.player, lock);
    }
}

fn main() {
    // DEBUG: MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
    // rysuje jedną klatkę, zapisuje do pliku i wychodzi (bez terminala).
    if let Ok(path) = std::env::var("MUSIC_FRAME_OUT") {
        debug_frame(&path);
        return;
    }

    let mode = match parse_mode() {
        Ok(m) => m,
        Err(()) => {
            print_help();
            std::process::exit(1);
        }
    };

    let _guard = TerminalGuard::new();

    let (info_tx, info_rx) = mpsc::sync_channel(1);
    {
        thread::spawn(move || player::info_worker(mode, info_tx));
    }
    let (art_tx, art_rx) = mpsc::channel::<(String, Option<art::Img>)>();
    // lrc_tty (bez --raw) to TUI: potrzebuje TTY, inaczej rysuje jedną pustą
    // klatkę i wychodzi — dlatego uruchamiamy je przez `script` (PTY) w tle,
    // a wątek czyta strumień zapisany do pliku i parsuje wiersze TUI
    // (zwykłe linie + podświetlona aktualna, ESC[7m).
    let mut info = Info::default();
    let out = std::process::Command::new("bins/matugen")
            .args(["image","/usr/share/cmake/Templates/Windows/ApplicationIcon.png","--prefer=saturation","--show-source-colors","--json","hex","--dry-run"])
            .output();
        if let Ok(o) = out {
            if o.status.success() {
                let hex = String::from_utf8_lossy(&o.stdout);
                info.accent = hex.lines().next().unwrap_or("#000000").trim().to_string();
            }
        }
    let mut loaded: Option<(String, art::Img)> = None;
    let mut wanted: Option<String> = None;
    let mut fetching = false;
    let mut last_fetch_fail: Option<Instant> = None;

    let mut expanded = false;
    let mut muted = false;
    let mut saved_vol = 0.6f64;

    let mut spin = 0.0f64;
    let mut seek_dir = 1.0f64;
    let mut seek_ts = Instant::now();
    let mut disc_texture = true;
    let mut arm_t = 0.0f64; // 0 = ramię odłożone, 1 = igła na płycie
    let mut last = Instant::now();
    let mut stdout = std::io::stdout();
    let mut screen = Screen::new(1, 1); // persistent — compose pisze tylko diffy
    let mut last_track_key = String::new(); // do wykrywania zmiany utworu → pełne odświeżenie

    loop {
        // 1. dane od wątka playerctl
        while let Ok(mut new_info) = info_rx.try_recv() {
            new_info.at = Some(Instant::now());
            if new_info.accent.is_empty() {
                // użyj okładki aktualnego utworu jeśli dostępna, inaczej domyślna
                let cover_path = if let Some((_, ref img)) = loaded {
                    let p = "/tmp/matugen_cover.png";
                    let _ = img.save(p);
                    p.to_string()
                } else {
                    "/usr/share/cmake/Templates/Windows/ApplicationIcon.png".to_string()
                };
                let out = std::process::Command::new("bins/matugen")
                    .args(["image", &cover_path, "--prefer=saturation", "--show-source-colors", "--json", "hex", "--dry-run"])
                    .output();
                if let Ok(o) = out {
                    if o.status.success() {
                        let hex = String::from_utf8_lossy(&o.stdout);
                        new_info.accent = hex.lines().next().unwrap_or("#000000").trim().to_string();
                    }
                }
            }
            wanted = new_info.art_url.clone();
            // stara okładka zostaje wyświetlona, dopóki nie dojdzie nowa
            info = new_info;
        }

        // 2. wyniki pobierania okładek
        while let Ok((url, img)) = art_rx.try_recv() {
            fetching = false;
            match img {
                Some(im) => {
                    if wanted.as_deref() == Some(url.as_str())
                        && loaded.as_ref().map(|(lu, _)| lu != &url).unwrap_or(true)
                    {
                        loaded = Some((url, im));
                    }
                }
                None => {
                    last_fetch_fail = Some(Instant::now());
                }
            }
        }

        // 3. zaplanuj pobranie okładki jeśli trzeba
        if !fetching {
            let need = wanted
                .as_ref()
                .map(|u| loaded.as_ref().map(|(lu, _)| lu != u).unwrap_or(true))
                .unwrap_or(false);
            let throttled = last_fetch_fail
                .map(|t| t.elapsed() < Duration::from_secs(5))
                .unwrap_or(false);
            if need && !throttled {
                fetching = true;
                art::fetch_async(wanted.clone().unwrap(), art_tx.clone());
            }
        }

        // 4. czas (animacja + interpolacja pozycji)
        let now = Instant::now();
        let dt = (now - last).as_secs_f64();
        last = now;
        let playing = info.status == "Playing";
        // spin: 2× wolniej przy pauzie
        if playing {
            spin += dt * seek_dir;
        } else {
            spin += dt * (seek_dir * 0.15);
        }
        // reset seek_dir po 1.0 s od ostatniej strzałki
        if seek_dir != 1.0 && now.duration_since(seek_ts).as_secs_f64() > 0.15 {
            seek_dir = 1.0;
        }
        // ramię płynnie podnosi się / opuszcza (animacja ~0.6 s)
        let arm_step = dt / 0.6;
        arm_t += (if playing { 1.0 } else { 0.0 } - arm_t).clamp(-arm_step, arm_step);
        let disp_pos = info.position
            + if playing {
                info.at.map(|t| now.duration_since(t).as_secs_f64()).unwrap_or(0.0)
            } else {
                0.0
            };

        // pełne odświeżenie ekranu przy zmianie utworu (eliminuje glitch)
        let current_key = format!("{}|{}|{}|{}",
            info.title.trim(), info.player, info.art_url.as_deref().unwrap_or(""), info.status);
        if current_key != last_track_key {
            last_track_key = current_key;
            let _ = execute!(stdout, Clear(ClearType::All));
            screen.reset();
        }

        // 5. render — jeden persistent Screen; przy resi różnice pełny fizyczny przerys
        let (tw, th) = terminal::size().unwrap_or((80, 24));
        if tw as usize != screen.w || th as usize != screen.h {
            let _ = execute!(stdout, Clear(ClearType::All));
            screen = Screen::new(tw as usize, th as usize);
        }
        let view = render::View {
            info: &info,
            expanded,
            muted,
            disp_pos,
            spin,
            arm_t,
                    disc_texture: disc_texture,
        };
        render::draw(&mut screen, &view, loaded.as_ref());
        let frame = screen.compose();
        let _ = stdout.write_all(frame.as_bytes());
        let _ = stdout.flush();

        // 6. klawiatura
        if event::poll(Duration::from_millis(8)).unwrap_or(false) {
            if let Ok(Event::Key(k)) = event::read() {
                if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c') if k.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                        KeyCode::Char('v') | KeyCode::Char('V') => {
                            if info.present {
                                let p = info.player.clone();
                                thread::spawn(move || player::send_action(mode, &p, "play-pause"));
                            }
                        }
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            if info.present {
                                let p = info.player.clone();
                                thread::spawn(move || player::send_action(mode, &p, "previous"));
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            if info.present {
                                let p = info.player.clone();
                                thread::spawn(move || player::send_action(mode, &p, "next"));
                            }
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            if info.present {
                                // głośność z ostatniego poll — bez synchronicznego volume_get
                                if !muted {
                                    saved_vol = if info.volume > 0.01 { info.volume } else { saved_vol };
                                    let p = info.player.clone();
                                    thread::spawn(move || player::volume_set(mode, &p, 0.0));
                                } else {
                                    let v = if saved_vol < 0.01 { 0.5 } else { saved_vol };
                                    let p = info.player.clone();
                                    thread::spawn(move || player::volume_set(mode, &p, v));
                                }
                                muted = !muted;
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if info.present {
                                let inf2 = info.clone();
                                thread::spawn(move || lock_current(mode, &inf2));
                            }
                        }
                        KeyCode::Char('z') | KeyCode::Char('Z') => {
                            expanded = !expanded;
                        }
                        KeyCode::Left => {
                            if info.present {
                                let p = info.player.clone();
                                thread::spawn(move || player::seek_rel(mode, &p, -1.0));
                                seek_dir = -2.0;
                                seek_ts = now;
                            }
                        }
                        KeyCode::Right => {
                            if info.present {
                                let p = info.player.clone();
                                thread::spawn(move || player::seek_rel(mode, &p, 1.0));
                                seek_dir = 2.0;
                                seek_ts = now;
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            disc_texture = !disc_texture;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
