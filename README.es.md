# music

Reproductor terminal estilo vinilo para MPRIS (Spotify, universal). Imagen de portada en tiempo real, cálculo de color dominante, animación de disco giratorio y control por teclado con `playerctl`.

---

## 1. Técnico / Instalación

**Lenguaje:** Rust `edition = 2021`  
**Dependencias:** `crossterm`, `image`, `ureq`

**Requisitos:**
- `playerctl` funcionando con tu reproductor (MPRIS) o Spotify instalado.
- Un terminal que soporte modo crudo (`raw mode`) y pantalla alternativa (`crossterm`).
- `bins/matugen` incluido: la aplicación lo usa en tiempo real para extraer colores dominantes de las portadas.

**Compilar:**
```bash
cargo build --release
```

**Binarios incluidos (`bins/`):**
- `matugen` — extrae color dominante de una imagen (usado por la interfaz para la paleta).
- `loop-control` / `loop-control-spotify` — scripts para bloquear / liberar el bucle del tema actual.
- `lrc_tty` — ayuda de visualización de letras.
- `figlet` / `3d.flf` — scripts de banners / arte ASCII.

**Uso:**
```bash
# Modo universal MPRIS
./target/release/music -u

# Solo Spotify
./target/release/music -s

# Ayuda
./target/release/music -h
```

**Estructura del proyecto:**
- `src/main.rs` — ciclo principal, renderizado, teclado, hilos.
- `src/player.rs` — interacción con `playerctl`, MPRIS / Spotify.
- `src/render.rs` — render en terminal, animación del disco y aguja.
- `src/art.rs` — descarga asíncrona de portadas, procesamiento gráfico.
- `screenshots/demo.mp4` — demo en video.

**Depuración (un solo frame, sin TTY):**
```bash
MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
```

**Atajos de teclado:**
| Tecla | Función |
|---|---|
| `V` | Reproducir / Pausar |
| `C` | Bloquear / Desbloquear bucle del tema actual |
| `B` | Tema anterior |
| `N` | Tema siguiente |
| `M` | Silenciar / Restaurar volumen |
| `Z` | Expandir / Colapsar panel de info |
| `D` | Alternar textura del disco |
| `Izquierda` / `Derecha` | Buscar atrás / adelante |
| `Q` | Salir |

**Modos:**
- `Universal` (`-u` / por defecto con argumentos): cualquier reproductor MPRIS a través de `playerctl`.
- `Spotify` (`-s`): solo Spotify con `loop-control-spotify`.

---

## 2. Qué es / Parte no técnica

`music` convierte tu terminal en un giradiscos minimalista. Escucha tu reproductor de música, descarga la portada del álbum, calcula un color dominante a partir de ella y renderiza un disco giratorio con una aguja que baja y sube con la reproducción. La animación es continua, se adapta al color de tu música, y solo necesita tu terminal y `playerctl` — nada de navegador ni interfaz gráfica adicional.

El diseño es deliberadamente minimalista: pantalla oscura, un solo disco, un panel de detalles que puedes abrir con `Z`. Está pensado para funcionar en un terminal dividido mientras trabajas, no para competir con aplicaciones de escritorio completas.

**Video de demostración (integrado):**

<video src="screenshots/demo.mp4" controls width="100%"></video>

Si el reproductor integrado no carga, abre `screenshots/demo.mp4` localmente.
