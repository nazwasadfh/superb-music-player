use std::io::Read;
use std::sync::mpsc::Sender;

pub type Img = image::RgbImage;
pub type Rgb = (u8, u8, u8);

/// Wątek pobiera okładkę i odsyła (url, image) kanałem. None = błąd pobierania.
pub fn fetch_async(url: String, tx: Sender<(String, Option<Img>)>) {
    let _ = std::thread::Builder::new()
        .name("art-fetch".into())
        .spawn(move || {
            let img = fetch_decode(&url);
            let _ = tx.send((url, img));
        });
}

fn fetch_decode(url: &str) -> Option<Img> {
    let bytes: Vec<u8> = if let Some(rest) = url.strip_prefix("file://") {
        let path = percent_decode(rest);
        std::fs::read(path).ok()?
    } else if url.starts_with("http://") || url.starts_with("https://") {
        let resp = ureq::get(url)
            .timeout(std::time::Duration::from_secs(8))
            .set("User-Agent", "music-cli/0.1")
            .call()
            .ok()?;
        let mut buf = Vec::new();
        resp.into_reader()
            .take(24 * 1024 * 1024)
            .read_to_end(&mut buf)
            .ok()?;
        buf
    } else {
        return None;
    };
    decode(&bytes)
}

fn decode(bytes: &[u8]) -> Option<Img> {
    let dynimg = image::load_from_memory(bytes).ok()?;
    Some(dynimg.to_rgb8())
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
