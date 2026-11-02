# music

Terminalowy odtwarzacz winylów dla MPRIS (Spotify, uniwersalny). Płyty, obliczanie dominującego koloru, animacja koła i igły z aktualnym obrazkiem, sterowanie klawiaturowe za pomocą `playerctl`.

---

## 1. Techniczne / Instalacja

**Język:** Rust `edition = 2021`  
**Zależności:** `crossterm`, `image`, `ureq`

**Wymagania:**
- `playerctl` działający z Twoim graczem (MPRIS) lub Spotify.
- Terminal obsługujący tryb napięty / alternatywny ekran (`crossterm`).
- `bins/matugen` znajduje się w repo — aplikacja wykorzystuje go w czasie rzeczywistym do wyciągania kolorów z obrazków albumowych.

**Budowanie:**
```bash
cargo build --release
```

**Narzędzia włączone (`bins/`):**
- `matugen` — wykrywa dominujący kolor obrazka (korzystane przez TUI do temowania).
- `loop-control` / `loop-control-spotify` — przydatne skrypty do ustawiania / odzyskiwania obecnego utworu w pętli.
- `lrc_tty` — pomocnicze narzędzie dla wyświetlacza tekstu ze słowami piosenki.
- `figlet` / `3d.flf` — identyfikatory / schrony ASCII.

**Użycie:**
```bash
# Uniwersalna obsługa MPRIS
./target/release/music -u

# Obsługa tylko Spotify
./target/release/music -s

# Pomoc / wersja
./target/release/music -h
```

**Rozszarzaniny kodu źródłowego:**
- `src/main.rs` — pętla zdarzeń, renderowanie, wejście klawiatury, zarządzanie wątkami.
- `src/player.rs` — interakcja z `playerctl`, komunikacja MPRIS / Spotify.
- `src/render.rs` — renderowanie terminala, mieszanie klatek, animacja koła i igły.
- `src/art.rs` — asynchroniczne pobieranie obrazków, przetwarzanie graficzne.
- `screenshots/demo.mp4` — nagranie demonstracyjne.

**Debug (jedna klatka, bez TTY):**
```bash
MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
```

**Klawisze klucza:**
| Klawisz | Funkcja |
|---|---|
| `V` | Play / Pause |
| `C` | Właczenie / wyłączenie obecnej pętli |
| `B` | Poprzedni utwór |
| `N` | Następny utwór |
| `M` | Umknij / Wróć głośność |
| `Z` | Rozwinij / Zwiń panel informacji |
| `D` | Przełączaj tekstura płyty |
| `Strzał w lewo` / `Strzał w prawo` | Poszukiwanie wstecznie / naprzód |
| `Q` | Zamknij |

**Tryby:**
- `Uniwersalny` (`-u` / domyślne z parametrami): dowolny gracz MPRIS za pośrednictwem `playerctl`.
- `Spotify` (`-s`): spolszczona obsługa tylko Spotify z użyciem `loop-control-spotify`.

---

## 2. Co to jest / Przypadkowe

`music` zamienia Twój terminal w minimalistyczny stół z winylami. Listenie Twojej muzyki, łapy nowy obrazek i obliczanie dominującego koloru z niego, renderuje koło z igłą, która opada i podnosi się razem z odtwaraniem. Animacja jest ciągła, dostosowana do Twojej muzyki i potrzebuje tylko terminala i `playerctl` — zero przeglądarek ani interfejsów GUI.

Design jest minimalistyczny: czarny ekran, pojedyncze koło, rozwijany panel ze szczegółami z obsługą `Z`. To narzędzie siedzi w dolonym splitterminalie podczas pracy, a nie konkuruje pełnymi desktopowymi aplikacjami mediowymi.

**Przegląd nagrania demo (wtłoczone):**

<video src="screenshots/demo.mp4" controls width="100%"></video>

Jeśli niektóra zawartość graficzna nie renderuje się, otwórz `screenshots/demo.mp4` lokalnie.
