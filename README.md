# Compare

A fast, lightweight, and modern side-by-side graphical diff tool built in Rust using [Iced](https://github.com/iced-rs/iced).

## Features

- **Side-by-Side Comparison**: Intuitive 2-column view with intra-line character-level diff highlighting.
- **Drag & Drop**: Drop two files together or one after the other onto the window or directly onto the macOS Dock icon.
- **Dynamic Sizing**: Starts in a compact launcher window (400×400) and automatically maximizes when files are loaded.
- **Customizable Theming & Colors**:
  - 22 built-in presets (Catppuccin, Nord, Dracula, Solarized, Tokyo Night, Gruvbox, etc.).
  - User-defined theme files (`<name>.toml`) to define new themes or override presets.
  - Fully customizable diff colors (not always just red and green—cyan/coral, teal/plum, blue/amber, etc.).
- **Line Wrapping**: Word-wrapped lines by default, with an instant toggle in the bottom toolbar between wrapped lines and independent horizontal scrolling.
- **Font Sizing & Zoom Shortcuts**:
  - Configurable default font size via `font_size` in `config.toml` (defaults to `13.0`).
  - `⌘ +` / `⌘ -` on macOS (`Ctrl +` / `Ctrl -` on Windows and Linux) to grow and shrink the font size of diff content and line numbers.
  - `⌘ 0` on macOS (`Ctrl 0` on Windows and Linux) to reset to the configured default font size.

---

## Configuration

Compare looks for `config.toml` in the following prioritized locations:

1. Directory set via `COMPARE_CONFIG_DIR` (or legacy `DIFFITRUST_CONFIG_DIR`) environment variable
2. Current working directory (`./config.toml`)
3. `~/.config/compare/config.toml` (or `~/.config/diffitrust/config.toml`)
4. `~/Library/Application Support/Compare/config.toml` (macOS)
5. `%APPDATA%\Compare\config.toml` (Windows)

### Quick Setup

To get started with a starter configuration on macOS or Linux:

```bash
mkdir -p ~/.config/compare
cp config.example.toml ~/.config/compare/config.toml
```

---

## How To: Themes

Compare offers flexible ways to theme your diff viewer:
1. **Automatic OS Theme Toggling**: Specify both a `light_theme` and a `dark_theme` in `config.toml`. Compare will automatically switch between them as your system appearance changes.
2. **Built-in Theme Presets**: Pick from 22 curated themes.
3. **Inline Diff Color Overrides**: Customize diff colors directly in `config.toml`.
4. **Custom Theme Files**: Create separate `.toml` theme files to share, organize, or override themes.

---

### 1. Automatic OS Light & Dark Theme Toggling (Recommended)

Set both `light_theme` and `dark_theme` in `config.toml`. Compare automatically detects your OS appearance on launch and dynamically switches whenever you change system dark/light mode:

```toml
# config.toml
light_theme = "kaleidescope"
dark_theme = "Nord"
```

Alternatively, you can use the `[theme]` table:

```toml
# config.toml
[theme]
light = "CatppuccinLatte"
dark = "CatppuccinMocha"
```

If you only want a single theme regardless of OS mode, you can still specify `preset = "..."` or `theme = "..."`.

#### Available Presets

| Category | Presets |
| :--- | :--- |
| **Dark Themes** | `CatppuccinMocha`, `CatppuccinMacchiato`, `CatppuccinFrappe`, `Nord`, `Dracula`, `TokyoNight`, `TokyoNightStorm`, `SolarizedDark`, `GruvboxDark`, `KanagawaWave`, `KanagawaDragon`, `Moonfly`, `Nightfly`, `Oxocarbon`, `Ferra`, `Dark` |
| **Light Themes** | `CatppuccinLatte`, `SolarizedLight`, `GruvboxLight`, `TokyoNightLight`, `KanagawaLotus`, `Light` |

---

### 2. Customizing Diff Colors (Not Just Red and Green)

You can customize individual diff elements in `config.toml` under the `[diff]` section. All colors accept hex values (`#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA`).

#### Diff Color Reference

| Key | Description | Example |
| :--- | :--- | :--- |
| `delete_bg` | Background of deleted / left-differing lines | `"#3a1520"` |
| `delete_highlight` | Background of intra-line character/word deletions | `"#5a2030"` |
| `delete_line_num` | Line number text color for deletions | `"#f38ba8"` |
| `insert_bg` | Background of inserted / right-differing lines | `"#183226"` |
| `insert_highlight` | Background of intra-line character/word insertions | `"#254e3c"` |
| `insert_line_num` | Line number text color for insertions | `"#a6e3a1"` |
| `equal_line_num` | Line number text color for identical lines | `"#6c7086"` |
| `text` | Diff content text color | `"#cdd6f4"` |
| `background` | Window and diff view background | `"#1e1e2e"` |
| `subheader_bg` | Subheader / file name bar background | `"#181825"` |
| `divider` | Border and divider line color | `"#313244"` |
| `empty_cell_bg` | Background of placeholder empty cells | `"#181825"` |

#### Example: Modern Cyan & Coral Diff Palette

```toml
# config.toml
preset = "Nord"

[diff]
# Coral / Crimson for deletions
delete_bg = "#3b222e"
delete_highlight = "#5c2e3d"
delete_line_num = "#bf616a"

# Cyan / Teal for insertions
insert_bg = "#1f3a3d"
insert_highlight = "#2b565a"
insert_line_num = "#88c0d0"

# Subtle line numbers & clean dividers
equal_line_num = "#4c566a"
subheader_bg = "#272c36"
divider = "#3b4252"
```

---

### 3. Creating Custom Theme Files

To keep your configuration modular or define standalone themes, place a `<name>.toml` file in the same directory as `config.toml`, or inside a `themes/` subfolder.

#### Step 1: Create the theme file

Create `~/.config/compare/themes/oceanic-diff.toml`:

```toml
name = "Oceanic Diff"
based_on = "Nord"       # Base preset to inherit UI controls and dark/light mode from

[diff]
delete_bg = "#3b222e"
delete_highlight = "#5c2e3d"
delete_line_num = "#bf616a"

insert_bg = "#1f3a3d"
insert_highlight = "#2b565a"
insert_line_num = "#88c0d0"

equal_line_num = "#4c566a"
text = "#eceff4"
background = "#2e3440"
subheader_bg = "#272c36"
divider = "#3b4252"
```

#### Step 2: Select your theme in `config.toml`

```toml
# config.toml
preset = "oceanic-diff"
```

---

### 4. Overriding Built-In Presets

You can override any built-in preset's diff colors without changing its name. Simply place a file named after the preset (e.g. `CatppuccinMocha.toml` or `Nord.toml`) in your config directory or `themes/` folder:

```toml
# ~/.config/compare/themes/CatppuccinMocha.toml
based_on = "CatppuccinMocha"

[diff]
delete_bg = "#432029"
delete_highlight = "#6a2d3b"
insert_bg = "#1e3a2f"
insert_highlight = "#2c5b47"
```

Whenever `preset = "CatppuccinMocha"` is selected, Compare automatically loads your custom file instead of the default preset colors.

---

## Building & Packaging

### 1. macOS

Build the native binary:
```bash
cargo build --release
```

To package a standalone macOS Application bundle (`Compare.app`) and DMG installer:
```bash
./scripts/build_macos.sh
```
The output `.app` and `.dmg` will be placed in `dist/`.

---

### 2. Linux

#### System Prerequisites
Compare uses [Iced](https://github.com/iced-rs/iced), which relies on system libraries for windowing, font rendering, and file dialogs.

- **Ubuntu / Debian / Linux Mint**:
  ```bash
  sudo apt-get update
  sudo apt-get install -y libxkbcommon-dev libfontconfig1-dev pkg-config libwayland-dev libgtk-3-dev
  ```

- **Fedora / RHEL**:
  ```bash
  sudo dnf install libxkbcommon-devel fontconfig-devel pkgconf-pkg-config wayland-devel gtk3-devel
  ```

- **Arch Linux / Manjaro**:
  ```bash
  sudo pacman -S libxkbcommon fontconfig pkg-config wayland gtk3
  ```

#### Build
```bash
cargo build --release
```
The executable is generated at `target/release/diffitrust`.

---

### 3. Windows

#### Prerequisites
- Rust with the `x86_64-pc-windows-msvc` target installed (`rustup default stable-x86_64-pc-windows-msvc`).
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload.

#### Build
```cmd
cargo build --release
```
The executable is generated at `target\release\diffitrust.exe`.

> [!NOTE]
> The release executable is configured with `#![windows_subsystem = "windows"]`, so no background command prompt / terminal window will appear when running Compare.

---

### 4. Cross-Compiling from macOS

If you develop on macOS and want to generate Windows or Linux binaries locally:

#### To Windows (using `cargo-xwin`)
[`cargo-xwin`](https://github.com/rust-cross/cargo-xwin) allows cross-compiling MSVC Windows binaries from macOS without requiring a Windows machine or MinGW:

```bash
# Install cargo-xwin and the MSVC target
cargo install cargo-xwin
rustup target add x86_64-pc-windows-msvc

# Build Windows executable
cargo xwin build --release --target x86_64-pc-windows-msvc
```
The resulting executable will be at `target/x86_64-pc-windows-msvc/release/diffitrust.exe`.

#### To Linux (using Docker or `cross`)
Because Linux GUI applications link against system C libraries (Fontconfig, Wayland/X11, etc.), using Docker or [`cross`](https://github.com/cross-rs/cross) is the cleanest method:

```bash
# Install cross
cargo install cross

# Build Linux binary via Docker container
cross build --release --target x86_64-unknown-linux-gnu
```
The resulting binary will be at `target/x86_64-unknown-linux-gnu/release/diffitrust`.

---

### 5. Automated Multi-Platform Releases (GitHub Actions)

A GitHub Actions workflow is provided at [`.github/workflows/build.yml`](.github/workflows/build.yml). When you push code or tags to GitHub, it automatically builds and packages:
- **Windows**: `compare-windows-x86_64.exe`
- **Linux**: `compare-linux-x86_64`
- **macOS**: `Compare.dmg` and `Compare.app`

Artifacts are downloadable directly from the GitHub Actions run summary or releases.
