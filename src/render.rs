use crate::art::{Img, Rgb};
use crate::player::Info;
use std::fmt::Write as _;

pub const PANEL_W: usize = 21;

const ACCENT: Rgb = (72, 214, 255);
fn accent_color(v: &View) -> Rgb {
    if !v.info.accent.is_empty() {
        parse_hex(&v.info.accent).unwrap_or(ACCENT)
    } else {
        ACCENT
    }
}
const KEY: Rgb = (120, 216, 255);
const GRAY: Rgb = (150, 150, 160);
const GRAY2: Rgb = (95, 95, 105);
const RIM: Rgb = (4, 4, 5); // niewidoczny — czarny obręb ukryty
const TITLE_C: Rgb = (245, 245, 250);
const ARTIST_C: Rgb = (170, 170, 185);
const ARM_M: Rgb = (185, 192, 200); // metal ramienia
const HEAD_C: Rgb = (28, 28, 32); // głowica
const PIVOT_C: Rgb = (75, 75, 82); // podstawka ramienia
const PIVOT_D: Rgb = (45, 45, 50); // centrum podstawki

const TAU: f64 = std::f64::consts::TAU;
const SPIN_PERIOD: f64 = 2.0; // sekundy na obrót

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: Option<char>,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
}

pub struct Screen {
    pub w: usize,
    pub h: usize,
    cells: Vec<Cell>,
    prev: Vec<Cell>, // poprzednia klatka — do pisania diffów (brak migotania)
}

impl Screen {
    pub fn new(w: usize, h: usize) -> Self {
        let blank = vec![
            Cell { ch: None, fg: None, bg: None };
            w * h
        ];
        Self {
            w,
            h,
            cells: blank.clone(),
            prev: blank,
        }
    }

    #[inline]
    pub fn set(&mut self, y: usize, x: usize, ch: Option<char>, fg: Option<Rgb>, bg: Option<Rgb>) {
        if y < self.h && x < self.w {
            self.cells[y * self.w + x] = Cell { ch, fg, bg };
        }
    }

    /// Tekst ucięty do `budget` kolumn; zwraca nową pozycję x.
    pub fn text_trunc(&mut self, y: usize, x: usize, budget: usize, s: &str, fg: Rgb) -> usize {
        let mut cx = x;
        for c in s.chars() {
            if cx >= x + budget || cx >= self.w {
                break;
            }
            self.set(y, cx, Some(c), Some(fg), None);
            cx += 1;
        }
        cx
    }

    /// Compose z diffem względem poprzedniej klatki — pisz wyłącznie
    /// zmienionych komórek (najedź kursor na każdej). Eliminuje blokowanie
    /// write na dużych ekranach i migotanie.
    /// Pełne odświeżenie — usuwa poprzednią klatkę, więc compose() pisze wszystko.
    pub fn reset(&mut self) {
        let blank = vec![Cell { ch: None, fg: None, bg: None }; self.w * self.h];
        self.prev = blank;
    }

    pub fn compose(&mut self) -> String {
        let mut out = String::with_capacity(self.w * self.h / 8 + 64);
        if self.prev == self.cells {
            return out;
        }
        let mut wfg: Option<Rgb> = None; // napisane kolory — persistent na terminalu
        let mut wbg: Option<Rgb> = None;
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                let c = self.cells[i];
                if self.prev[i] == c {
                    continue;
                }
                let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
                if c.fg != wfg {
                    match c.fg {
                        Some((r, g, b)) => {
                            let _ = write!(out, "\x1b[38;2;{};{};{}m", r, g, b);
                        }
                        None => out.push_str("\x1b[39m"),
                    }
                    wfg = c.fg;
                }
                if c.bg != wbg {
                    match c.bg {
                        Some((r, g, b)) => {
                            let _ = write!(out, "\x1b[48;2;{};{};{}m", r, g, b);
                        }
                        None => out.push_str("\x1b[49m"),
                    }
                    wbg = c.bg;
                }
                match c.ch {
                    Some(ch) => out.push(ch),
                    None => out.push(' '),
                }
            }
        }
        self.prev.clone_from(&self.cells);
        out
    }
}

pub struct View<'a> {
    pub info: &'a Info,
    pub expanded: bool,
    pub muted: bool,
    pub disp_pos: f64,
    pub spin: f64,
    pub arm_t: f64,

    pub disc_texture: bool,
}

/// Promień tarczy i jej środek (w kolumnach/wierszach terminala).
pub fn disc_geometry(w: usize, h: usize) -> (f64, f64, f64) {
    let rows = h.saturating_sub(3);
    let pw = PANEL_W.min(w);
    let left_margin = 74; // płyta przesunięta w prawo; lewa strona 40 znaków
    let disc_w = w.saturating_sub(pw + left_margin + 2);
    let r = ((disc_w as f64 / 2.0).min(rows as f64 - 3.0) - 1.0).max(0.0);
    let cx = left_margin as f64 + (disc_w as f64 / 2.0);
    let cy = rows as f64 / 2.0;
    (r, cx, cy)
}

fn quant(v: f64) -> u8 {
    // kwantyzacja kanału (mniejszy strumień ANSI)
    ((v / 12.0).round() * 12.0 + 6.0).clamp(0.0, 255.0) as u8
}

fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Pozycja ramienia gramofonu. Igła opada na płytę podczas gry,
/// a zjeżdża do wewnętrznych rowków w miarę postępu utworu.
/// CAŁA geometria jest w jednostkach WZGLĘDEM środka płyty (dx, dy —
/// dy w podwójnych jednostkach pionu `▀`), więc pozycja nie zależy
/// od szerokości okna, a zawartość ramienia zawsze trafia w tarczę.
pub struct Arm {
    /// podstawka ramienia (względem środka płyty)
    pub piv: (f64, f64),
    /// głowica (względem środka płyty)
    pub head: (f64, f64),
    pub head_r: f64, // promień głowicy (jednostki)
    pub width: f64,  // półszerokość ramienia
    pub base_r: f64, // promień podstawki
}

fn arm_geometry(r: f64, cx: f64, cy: f64, frac: f64, arm_t: f64) -> Arm {
    let row_units = cy * 2.0; // wysokość obszaru płyty (w jednostkach)
    let head_r = (1.5 + 0.012 * r).clamp(1.5, 2.4);
    let width = (0.75 + 0.006 * r).clamp(0.75, 1.15);
    let base_r = (2.0 + 0.02 * r).clamp(2.0, 3.4);

    // igła na płycie zależnie od postępu — od brzegu prawej strony do środka
    // (jak w prawdziwym gramofonie, gdzie ramię nie przecina osi płyty)
    let hr = ((r * (0.85 - 0.55 * frac)).min(r - 1.8)).max(0.3 * r);
    let ang_h = -50f64.to_radians();
    let on = (hr * ang_h.cos(), hr * ang_h.sin());

    // parkowanie w pustym narożniku nad prawym górnym brzegiem tarczy
    // (odległość > r przy każdej proporcji okna)
    let ang_o = -45f64.to_radians();
    let d_o = 1.15 * r;
    let off = (d_o * ang_o.cos(), d_o * ang_o.sin());

    // podstawa: na zewnątrz płyty, w prawym dolnym rogu
    let piv_dy = (0.85 * r).min((row_units - 5.0).max(0.3 * r));
    let mut piv_dx = (((1.45 * r) * (1.45 * r) - piv_dy * piv_dy)).sqrt().max(0.6 * r);
    // nigdy poza prawą krawędź obszaru płyty (podstawka mieści się w całości)
    piv_dx = piv_dx.min((cx - base_r - 0.5).max(0.6 * r));
    let piv = (piv_dx, piv_dy);

    let s = smooth(arm_t);
    let head = (off.0 + (on.0 - off.0) * s, off.1 + (on.1 - off.1) * s);

    Arm {
        piv,
        head,
        head_r,
        width,
        base_r,
    }
}

/// Nakładka ramienia w punkcie (jednostki: dx,dy względem środka płyty).
fn arm_at(dx: f64, dy: f64, arm: &Arm, playing: bool, v: &View) -> Option<Rgb> {
    // głowica z igłą
    let dh = ((dx - arm.head.0) * (dx - arm.head.0) + (dy - arm.head.1) * (dy - arm.head.1)).sqrt();
    if dh < arm.head_r {
        if dh < arm.head_r * 0.45 {
            return Some(if playing { accent_color(v) } else { (70, 70, 76) });
        }
        return Some(HEAD_C);
    }
    // ramię: odcinek pivot -> head
    let abx = arm.head.0 - arm.piv.0;
    let aby = arm.head.1 - arm.piv.1;
    let ab2 = abx * abx + aby * aby;
    if ab2 > 0.001 {
        let t = (((dx - arm.piv.0) * abx + (dy - arm.piv.1) * aby) / ab2).clamp(0.0, 1.0);
        let qx = arm.piv.0 + abx * t;
        let qy = arm.piv.1 + aby * t;
        let dq = ((dx - qx) * (dx - qx) + (dy - qy) * (dy - qy)).sqrt();
        if dq < arm.width {
            return Some(ARM_M);
        }
    }
    // podstawka ramienia
    let dp = ((dx - arm.piv.0) * (dx - arm.piv.0) + (dy - arm.piv.1) * (dy - arm.piv.1)).sqrt();
    if dp < arm.base_r {
        return Some(if dp < arm.base_r * 0.45 { PIVOT_D } else { PIVOT_C });
    }
    None
}

/// Bilinearne próbkowanie obrazka w punkcie (fx, fy).
fn bilinear(img: &Img, fx: f64, fy: f64) -> (f64, f64, f64) {
    let w = img.width();
    let h = img.height();
    let x0 = fx.floor().clamp(0.0, (w - 1) as f64) as u32;
    let y0 = fy.floor().clamp(0.0, (h - 1) as f64) as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = (fx - x0 as f64).clamp(0.0, 1.0);
    let ty = (fy - y0 as f64).clamp(0.0, 1.0);
    let g = |x: u32, y: u32| {
        let p = img.get_pixel(x, y);
        (p[0] as f64, p[1] as f64, p[2] as f64)
    };
    let (a1, a2, a3) = g(x0, y0);
    let (b1, b2, b3) = g(x1, y0);
    let (c1, c2, c3) = g(x0, y1);
    let (d1, d2, d3) = g(x1, y1);
    let l = |a: f64, b: f64, t: f64| a + (b - a) * t;
    (
        l(l(a1, b1, tx), l(c1, d1, tx), ty),
        l(l(a2, b2, tx), l(c2, d2, tx), ty),
        l(l(a3, b3, tx), l(c3, d3, tx), ty),
    )
}

/// Kolor jednej połówki komórki na tarczy. Okładka jest wycinana w koło
/// i obracana: próbka mapowana biegunowo względem kąta phi (czas -> obrót).

/// Czarna obręcz z rotującą teksturą rowków — czarne wygląda jak część wirującej płyty.
fn rim_texture(r: f64, phi: f64, disc_texture: bool) -> Rgb {
    if !disc_texture {
        return RIM;
    }
    let ridge = 0.5 + 0.5 * (r * 3.2 + phi * 8.0).sin().clamp(-1.0, 1.0);
    let val = (4.0 + 16.0 * ridge) as u8;
    (val, val, val + 1)
}

fn cell_color(
    img: Option<&Img>,
    r: f64,
    dx: f64,
    dy: f64,
    big_r: f64,
    cos_p: f64,
    sin_p: f64,
    phi: f64,
    frac: f64,
    disc_texture: bool,
    v: &View,
) -> Rgb {
    // obręcz zewnętrzna: czarny rant z rotującą teksturą, potem pasek płyty
    if r > big_r {
        if r - big_r < 0.8 {
            return rim_texture(r, phi, disc_texture);
        }
        if r - big_r < 2.5 {
            let theta = dy.atan2(dx);
            let mut rel = theta - (-std::f64::consts::FRAC_PI_2);
            rel = rel.rem_euclid(TAU);
            let in_arc = frac > 0.0 && rel <= frac * TAU + 0.15;
            if in_arc {
                return accent_color(v);
            }
            return rim_texture(r, phi, disc_texture);
        }
    }

    // obręcz: pasek postępu albo czarny rant z rotującą teksturą
    if big_r - r < 0.8 {
        return rim_texture(r, phi, disc_texture);
    }
    if big_r - r < 1.5 {
        let theta = dy.atan2(dx);
        let mut rel = theta - (-std::f64::consts::FRAC_PI_2);
        rel = rel.rem_euclid(TAU);
        let in_arc = frac > 0.0 && rel <= frac * TAU + 0.15;
        if in_arc {
            return accent_color(v);
        }
        return rim_texture(r, phi, disc_texture);
    }

    // otwór w środku płyty (jak w prawdziwym winylu) + az połyskiem wokół otworu
    if r < 2.0 {
        return (4, 4, 5);
    }
    let hole_dim = if r < 3.5 { 0.55 } else { 1.0 };

    match img {
        Some(img) => {
            // obrót próbki: obrazek kręci się z tarczą
            let xr = dx * cos_p + dy * sin_p;
            let yr = -dx * sin_p + dy * cos_p;
            let w = img.width() as f64;
            let h = img.height() as f64;
            let fx = ((xr / big_r) * 0.5 + 0.5) * (w - 1.0);
            let fy = ((yr / big_r) * 0.5 + 0.5) * (h - 1.0);
            let (cr, cg, cb) = bilinear(img, fx, fy);
            // tekstura rowków winylowych — drobne radialne przyciemnienie
            let groove_f = if disc_texture {
                0.92 + 0.08 * (r * 3.2 + phi * 8.0).sin().clamp(-1.0, 1.0)
            } else {
                1.0
            };
            let (cr, cg, cb) = (cr * groove_f, cg * groove_f, cb * groove_f);
            // subtelna winieta przy krawędzi — daje efekt krążka
            let edge = big_r - r;
            let mut vig = if edge < 1.5 {
                0.55 + 0.45 * (edge / 1.5)
            } else {
                1.0
            };
            vig *= hole_dim;
            (quant(cr * vig), quant(cg * vig), quant(cb * vig))
        }
        None => {
            // brak okładki: klasyczny winyl z rowkami i odblaskiem
            let theta = dy.atan2(dx);
            let groove = 0.5 + 0.5 * (r * 2.6).sin();
            let mut b = 0.020 + 0.050 * groove;
            let ang = theta - phi - r * 0.28;
            let s = ang.cos().max(0.0);
            let s4 = s * s * s * s;
            let ang_b = ang + std::f64::consts::PI;
            let sb = ang_b.cos().max(0.0);
            let sb6 = sb * sb * sb * sb * sb * sb;
            b += 0.85 * s4 + 0.22 * sb6;
            let v = ((b.clamp(0.0, 1.0) * 15.0).round() / 15.0 * 255.0) as u8;
            (v, v, (v + 6).min(255))
        }
    }
}

fn draw_disc(s: &mut Screen, v: &View, img: Option<&Img>, r: f64, cx: f64, cy: f64) {
    if r < 4.0 {
        return;
    }
    let rows = s.h.saturating_sub(3);
    let disc_w = ((cx * 2.0).ceil() as usize).min(s.w);

    let phi = v.spin * TAU / SPIN_PERIOD;
    let (cos_p, sin_p) = (phi.cos(), phi.sin());
    let frac = if v.info.length_sec > 0.5 {
        (v.disp_pos / v.info.length_sec).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ramię gramofonu
    let playing = v.info.status == "Playing";
    let arm = arm_geometry(r, cx, cy, frac, v.arm_t);
    let (minx, maxx, miny, maxy) = (
        arm.piv.0.min(arm.head.0) - 3.5,
        arm.piv.0.max(arm.head.0) + 3.5,
        arm.piv.1.min(arm.head.1) - 3.5,
        arm.piv.1.max(arm.head.1) + 3.5,
    );

    for y in 0..rows {
        let dyt = (y as f64 * 2.0 + 0.5) - cy * 2.0;
        let dyb = dyt + 1.0;
        for x in 0..disc_w {
            let dx = x as f64 + 0.5 - cx;
            // połówka: kolor płyty + nakładka ramienia
            let sample = |dy: f64| -> Option<Rgb> {
                let mut c = {
                    let rr = (dx * dx + dy * dy).sqrt();
                    if rr > r + 3.0 {
                        None
                    } else {
                        Some(cell_color(img, rr, dx, dy, r, cos_p, sin_p, phi, frac, v.disc_texture, v))
                    }
                };
                if dx > minx && dx < maxx && dy > miny && dy < maxy {
                    if let Some(a) = arm_at(dx, dy, &arm, playing, v) {
                        c = Some(a);
                    }
                }
                c
            };
            let top = sample(dyt);
            let bot = sample(dyb);
            if top.is_none() && bot.is_none() {
                continue;
            }
            match (top, bot) {
                (Some(t), Some(b)) => s.set(y, x, Some('▀'), Some(t), Some(b)),
                (Some(t), None) => s.set(y, x, Some('▀'), Some(t), None),
                (None, Some(b)) => s.set(y, x, Some('▄'), Some(b), None),
                (None, None) => {}
            }
        }
    }
}

fn draw_panel(s: &mut Screen, v: &View) {
    let w = s.w;
    let h = s.h;
    let px = w.saturating_sub(PANEL_W);
    let limit = h.saturating_sub(4); // nie wchodź w pasek na dole

    let mut y = 1usize;
    // [Z] show keybinds
    if y < limit {
        let mut x = s.text_trunc(y, px, PANEL_W, "[", GRAY2);
        x += s.text_trunc(y, x, PANEL_W, "Z", KEY) - x;
        x += s.text_trunc(y, x, PANEL_W, "] ", GRAY2) - x;
        s.text_trunc(y, x, PANEL_W, "show keybinds", GRAY);
    }

    if v.expanded {
        let items: [(&str, &str); 10] = [
            ("V", "play-pause"),
            ("C", "lock-current"),
            ("B", "prev-track (← rewind)"),
            ("N", "next-track (→ forward)"),
            ("←", "rewind -10s (spin reverse)"),
            ("→", "forward +10s (spin faster)"),
            ("M", "mute"),
            ("L", "3D text (figlet) toggle"),
            ("D", "vinyl texture"),
            ("Q", "quit"),
        ];
        for (k, desc) in items {
            y += 1;
            if y >= limit {
                break;
            }
            let mut x = s.text_trunc(y, px, PANEL_W, "[", GRAY2);
            x += s.text_trunc(y, x, PANEL_W, k, KEY) - x;
            x += s.text_trunc(y, x, PANEL_W, "] ", GRAY2) - x;
            s.text_trunc(y, x, PANEL_W, desc, GRAY);
        }
        y += 2; // pusta linia przed statusem
    } else {
        y += 2;
    }

    // status
    let vals: [(&str, String); 3] = [
        ("player", v.info.player.clone()),
        (
            "loop",
            match v.info.loop_status.as_str() {
                "Track" => "LOCKED".to_string(),
                "None" => "unlocked".to_string(),
                "Playlist" => "playlist".to_string(),
                "" => String::new(),
                other => other.to_string(),
            },
        ),
        (
            "vol",
            if v.muted {
                "MUTED".to_string()
            } else {
                format!("{:.0}%", v.info.volume * 100.0)
            },
        ),
    ];
    for (label, val) in vals {
        if y >= limit {
            break;
        }
        let mut x = s.text_trunc(y, px, PANEL_W, label, GRAY2);
        x += s.text_trunc(y, x, PANEL_W, " ", GRAY2) - x;
        let vcolor = if val == "LOCKED" {
            ACCENT
        } else if val == "MUTED" {
            (255, 140, 140)
        } else {
            GRAY
        };
        s.text_trunc(y, x, PANEL_W.saturating_sub(x - px).min(13), &val, vcolor);
        y += 1;
    }
}


fn draw_bottom(s: &mut Screen, v: &View) {
    let w = s.w;
    let h = s.h;
    let ty = h - 1;

    // tytuł — autor
    let mut x = 2usize;
    match v.info.status.as_str() {
        "Playing" => {
            x += s.text_trunc(ty, x, w, "▶ ", accent_color(v)) - x;
        }
        "Paused" => {
            x += s.text_trunc(ty, x, w, "∥ ", GRAY) - x;
        }
        _ => {
            x += s.text_trunc(ty, x, w, "○ ", GRAY2) - x;
        }
    }
    x += s.text_trunc(ty, x, w, "♪ ", GRAY) - x;

    let budget = w.saturating_sub(x + PANEL_W.min(w) + 1);
    let mut bx = x;
    if !v.info.title.is_empty() {
        bx += s.text_trunc(ty, bx, budget, &v.info.title, TITLE_C) - bx;
        if !v.info.artist.is_empty() && bx < x + budget.saturating_sub(4) {
            bx += s.text_trunc(ty, bx, budget, " — ", GRAY2) - bx;
            s.text_trunc(ty, bx, budget, &v.info.artist, ARTIST_C);
        }
    } else if !v.info.album.is_empty() {
        s.text_trunc(ty, bx, budget, &v.info.album, ARTIST_C);
    }
}

#[allow(dead_code)]
fn wrap_words(s: &str, max_chars: usize) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in words {
        if cur.len() + 1 + w.len() > max_chars && !cur.is_empty() {
            lines.push(cur);
            cur = w.to_string();
        } else {
            if !cur.is_empty() { cur.push(' '); }
            cur.push_str(w);
        }
    }
    if !cur.is_empty() { lines.push(cur); }
    lines.join("\n")
}


fn parse_hex(s: &str) -> Option<Rgb> {
    let s = s.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()? as u32;
    let g = u8::from_str_radix(&s[2..4], 16).ok()? as u32;
    let b = u8::from_str_radix(&s[4..6], 16).ok()? as u32;
    Some((r as u8, g as u8, b as u8))
}

fn normalize_pl(s: &str) -> String {
    s.replace('ż', "z").replace('Ż', "Z")
     .replace('ą', "a").replace('Ą', "A")
     .replace('ę', "e").replace('Ę', "E")
     .replace('ó', "o").replace('Ó', "O")
     .replace('ś', "s").replace('Ś', "S")
     .replace('ł', "l").replace('Ł', "L")
     .replace('ź', "z").replace('Ź', "Z")
     .replace('ć', "c").replace('Ć', "C")
     .replace('ń', "n").replace('Ń', "N")
}


fn strip_feat(s: &str) -> String {
    // usuń (feat. ...) w tym z &
    let mut out = s.to_string();
    if let Some(start) = out.to_lowercase().find("(feat.") {
        let idx = if start >= 1 && out.as_bytes()[start-1] == b' ' { start - 1 } else { start };
        if let Some(end) = out[idx..].find(')') {
            out = format!("{}{}", &out[..idx], &out[idx+end+1..]);
            // przycinamy białe znaki z końcówek
            out = out.trim().to_string();
        }
    }
    out
}

fn draw_left_text(s: &mut Screen, v: &View) {
    let left_max = 90;
    // Wyczyść poprzedni tytuł przed rysowaniem — zapobiega glitchom przy krótszym tytule
    for y in 11..(s.h.min(46)) {
        for x in 2..left_max.min(s.w.saturating_sub(1)) {
            s.set(y, x, None, None, None);
        }
    }
    let title_raw = format!("{}", v.info.title.trim());
    let title_stripped = strip_feat(&title_raw);
    let title = normalize_pl(&title_stripped);
    // 3D tytuł z figlet (pełne 70-width, wiele linii)
    let fig_text = match std::process::Command::new("./bins/figlet")
        .args(["-f","bins/3d.flf","-w","90", &title]).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => title.clone()
    };
    let lines_fig: Vec<&str> = fig_text.split('\n').collect();
    let accent = if !v.info.accent.is_empty() {
        parse_hex(&v.info.accent).unwrap_or(ACCENT)
    } else {
        ACCENT
    };
    for (i, ln) in lines_fig.iter().enumerate() {
        if i >= 35 { break; }
        for (j, c) in ln.chars().enumerate() {
            let cx = 2 + j;
            if cx >= left_max { break; }
            s.set(11 + i, cx, Some(c), Some(accent), None);
        }
    }
    // ramka zaraz pod tytułem (tytuły kończą się na różnej wysokości) — do samego dołu
    let title_lines = lines_fig.len().min(35);
    let start_y = 11 + title_lines + 1; // jedna pusta linia między tytułem a ramką
    let bottom_y = s.h.saturating_sub(1);
    if start_y < bottom_y {
        let x0 = 2usize;
        let x1 = (x0 + left_max - 1).min(s.w.saturating_sub(1));
        let frame_c = GRAY2;
        // górna krawędź
        s.set(start_y, x0, Some('┌'), Some(frame_c), None);
        for x in (x0 + 1)..x1 {
            s.set(start_y, x, Some('─'), Some(frame_c), None);
        }
        s.set(start_y, x1, Some('┐'), Some(frame_c), None);
        // boki
        for y in (start_y + 1)..bottom_y {
            s.set(y, x0, Some('│'), Some(frame_c), None);
            s.set(y, x1, Some('│'), Some(frame_c), None);
        }
        // dolna krawędź (do samego dołu)
        s.set(bottom_y, x0, Some('└'), Some(frame_c), None);
        for x in (x0 + 1)..x1 {
            s.set(bottom_y, x, Some('─'), Some(frame_c), None);
        }
        s.set(bottom_y, x1, Some('┘'), Some(frame_c), None);
        // ramka pusta — bez lyrics
    }
    s.text_trunc(9, 2, left_max, "[D] vinyl texture  ·  [Z] bindy", GRAY);
    if !v.info.accent.is_empty() {
        s.text_trunc(10, 2, left_max, &format!("matugen accent: {}", v.info.accent), (140, 170, 190));
    }
}
fn draw_waiting(s: &mut Screen, cx: f64, cy: f64) {
    let mid = (cy as usize).max(1);
    let msg = "waiting for a player...";
    let x = (cx as usize).saturating_sub(msg.chars().count() / 2);
    s.text_trunc(mid, x, s.w, msg, GRAY2);
    let msg2 = "check playerctl / start your media player";
    let x2 = (cx as usize).saturating_sub(msg2.chars().count() / 2);
    s.text_trunc(mid + 1, x2, s.w, msg2, (70, 70, 78));
}

pub fn draw(s: &mut Screen, v: &View, loaded: Option<&(String, Img)>) {
    let w = s.w;
    let h = s.h;
    if w < 30 || h < 6 {
        return;
    }

    // tekst po lewej przed płytą
    draw_left_text(s, v);

    let (r, cx, cy) = disc_geometry(w, h);
    let img = loaded.map(|l| &l.1);

    if v.info.present {
        draw_disc(s, v, img, r, cx, cy);
    } else {
        draw_waiting(s, cx, cy);
    }

    draw_panel(s, v);
    draw_bottom(s, v);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Igła ma leżeć NA płycie podczas gry — niezależnie od rozmiaru okna
    /// (na pełnym ekranie tarczę ogranicza wysokość, przy wąskim — szerokość).
    /// Współrzędne ramienia są względne wobec środka płyty.
    #[test]
    fn igla_na_plycie_gdy_gra() {
        for (w, h) in [(220usize, 60usize), (120, 40), (60, 24), (191, 53)] {
            let (r, cx, cy) = disc_geometry(w, h);
            let a = arm_geometry(r, cx, cy, 0.1, 1.0);
            let d_on = (a.head.0 * a.head.0 + a.head.1 * a.head.1).sqrt();
            println!(
                "{}x{}: r={:.1} cx={:.1} cy={:.1} -> d_on={:.1}, head=({:.1},{:.1})",
                w, h, r, cx, cy, d_on, a.head.0, a.head.1
            );
            assert!(d_on < r - 0.5, "igła ma być na płycie podczas gry ({}x{})", w, h);
        }
    }

    /// Przy pauzie głowica ma być wyraźnie poza krążkiem i zmieścić się
    /// w całości w obszarze płyty (nie można jej uciąć na krawędzi).
    #[test]
    fn odlozone_ramie_poza_plyta() {
        for (w, h) in [(220usize, 60usize), (120, 40), (60, 24), (191, 53)] {
            let (r, cx, cy) = disc_geometry(w, h);
            let disc_w = (cx * 2.0).min(w as f64);
            let a = arm_geometry(r, cx, cy, 0.1, 0.0);
            let d_off = (a.head.0 * a.head.0 + a.head.1 * a.head.1).sqrt();
            println!(
                "{}x{}: r={:.1} -> d_off={:.1}, head=({:.1},{:.1})",
                w, h, r, d_off, a.head.0, a.head.1
            );
            assert!(d_off > r + 0.8, "głowica ma być poza płytą przy pauzie ({}x{})", w, h);
            assert!(
                cx + a.head.0 + a.head_r < disc_w && cx + a.head.0 - a.head_r > 0.0,
                "głowica nie może wystawać poza obszar płyty w poziomie ({}x{})",
                w,
                h
            );
            assert!(cy + a.head.1 / 2.0 > 0.0, "głowica nie może uciec ponad górę ({}x{})", w, h);
        }
    }

    /// Podstawka ramienia też ma się mieścić w obszarze płyty.
    #[test]
    fn podstawka_w_obszarze_plyty() {
        for (w, h) in [(220usize, 60usize), (120, 40), (60, 24), (191, 53)] {
            let (r, cx, cy) = disc_geometry(w, h);
            let disc_w = (cx * 2.0).min(w as f64);
            let a = arm_geometry(r, cx, cy, 0.1, 1.0);
            println!(
                "{}x{}: piv=({:.1},{:.1}) base_r={:.1}",
                w, h, a.piv.0, a.piv.1, a.base_r
            );
            assert!(
                cx + a.piv.0 + a.base_r < disc_w && cx + a.piv.0 - a.base_r > 0.0,
                "podstawka nie może wystawać poza obszar płyty w poziomie ({}x{})",
                w,
                h
            );
        }
    }

    #[test]
    fn nakladka_ramienia_zwraca_kolor() {
        let arm = arm_geometry(36.0, 49.0, 18.5, 0.1, 1.0);
        // środek głowicy
        assert!(arm_at(arm.head.0, arm.head.1, &arm, true, v).is_some());
        // podstawa
        assert!(arm_at(arm.piv.0 + 2.0, arm.piv.1 + 1.5, &arm, true).is_some());
        // środek odcinka ramienia
        let mid = ((arm.piv.0 + arm.head.0) / 2.0, (arm.piv.1 + arm.head.1) / 2.0);
        assert!(arm_at(mid.0, mid.1, &arm, true, v).is_some());
        // daleko od ramienia
        assert!(arm_at(0.0, -100.0, &arm, true, v).is_none());
    }

    #[test]
    fn dlugosc_ramienia_powiekszenie() {
        // ramię ma być wyraźnie dłuższe niż promień płyty (na pełnym ekranie > 2*r)
        let arm = arm_geometry(56.0, 99.0, 28.5, 0.1, 1.0);
        let len = ((arm.piv.0 - arm.head.0).powi(2) + (arm.piv.1 - arm.head.1).powi(2)).sqrt();
        println!("arm length at r=56: {:.1}", len);
        assert!(len > 1.5 * 56.0, "ramię powinno być wyraźnie dłuższe niż promień płyty");
    }
}
