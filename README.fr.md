# music

Lecteur terminal vinyle pour MPRIS (Spotify, universel). Art de couverture en temps réel, calcul de la couleur dominante, animation du disque tournant et contrôle clavier via `playerctl`.

---

## 1. Technique / Installation

**Langage :** Rust `edition = 2021`  
**Dépendances :** `crossterm`, `image`, `ureq`

**Exigences :**
- `playerctl` fonctionnel avec votre lecteur (MPRIS) ou Spotify installé.
- Un terminal prenant en charge le mode brut et l'écran alternatif (`crossterm`).
- `bins/matugen` inclus dans le dépôt ; l'application l'appelle au runtime pour extraire la couleur dominante de la couverture.

**Compilation :**
```bash
cargo build --release
```

**Binaires inclus (`bins/`) :**
- `matugen` — extrait la couleur dominante d'une image (utilisé par l'interface pour le thème).
- `loop-control` / `loop-control-spotify` — scripts d'aide pour verrouiller / déverrouiller la boucle du morceau actuel.
- `lrc_tty` — aide d'affichage des paroles.
- `figlet` / `3d.flf` — scripts d'ASCII / bannières.

**Usage :**
```bash
# Mode universel MPRIS
./target/release/music -u

# Mode Spotify uniquement
./target/release/music -s

# Aide
./target/release/music -h
```

**Fichiers du projet :**
- `src/main.rs` — boucle d'événements, rendu, clavier, gestion des threads.
- `src/player.rs` — interaction `playerctl`, MPRIS / Spotify.
- `src/render.rs` — rendu terminal, composition des images, animation du disque.
- `src/art.rs` — récupération asynchrone des couvertures, traitement d'image.
- `screenshots/demo.mp4` — vidéo de démonstration.

**Débogage (une seule image, sans TTY) :**
```bash
MUSIC_FRAME_OUT=/tmp/frame.ans ./target/release/music -u
```

**Raccourcis clavier :**
| Touche | Action |
|---|---|
| `V` | Lecture / Pause |
| `C` | Verrouiller / Déverrouiller la boucle du morceau actuel |
| `B` | Morceau précédent |
| `N` | Morceau suivant |
| `M` | Muet / Restaurer le volume |
| `Z` | Déplier / Replier le panneau d'informations |
| `D` | Basculer la texture du disque |
| `Gauche` / `Droite` | Rechercher en arrière / avant |
| `Q` | Quitter |

**Modes :**
- `Universel` (`-u` / par défaut avec arguments) : tout lecteur MPRIS via `playerctl`.
- `Spotify` (`-s`) : Spotify uniquement avec `loop-control-spotify`.

---

## 2. Ce que c'est / Partie non technique

`music` transforme votre terminal en un tourne-disques minimaliste. Il écoute votre lecteur, récupère la couverture actuelle, calcule une couleur d'accent à partir de celle-ci et affiche un disque tournant avec une aiguille qui abaisse et se lève avec la lecture. L'animation est continue, colorée selon votre musique, et ne demande qu'un terminal et `playerctl` — pas de navigateur ni d'interface graphique supplémentaire.

Le design est volontairement épuré : écran sombre, un seul disque, un panneau d'informations extensible avec `Z`. Il est destiné à s'installer dans un terminal partagé pendant que vous travaillez, sans concurrencer les applications multimédias complètes.

**Vidéo de démonstration (intégrée) :**

<video src="screenshots/demo.mp4" controls width="100%"></video>

Si la lecture intégrée ne fonctionne pas, ouvrez `screenshots/demo.mp4` localement.
