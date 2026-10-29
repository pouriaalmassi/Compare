use iced::{Color, Theme};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Diff-specific visual colors.
#[derive(Debug, Clone)]
pub struct DiffTheme {
    pub delete_bg: Color,
    pub delete_highlight: Color,
    pub delete_line_num: Color,
    pub insert_bg: Color,
    pub insert_highlight: Color,
    pub insert_line_num: Color,
    pub equal_line_num: Color,
    pub text: Color,
    pub background: Color,
    pub subheader_bg: Color,
    pub divider: Color,
    pub empty_bg: Color,
    pub empty_cell_bg: Color,
    pub button_bg: Color,
    pub button_border: Color,
}

/// The complete application theme including iced theme and diff colors.
#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub iced_theme: Theme,
    pub diff: DiffTheme,
    pub is_dark: bool,
    pub warning: Option<String>,
}

impl Default for AppTheme {
    fn default() -> Self {
        load_theme()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ThemeSetting {
    Name(String),
    Pair {
        light: Option<String>,
        dark: Option<String>,
    },
}

/// Structure of `config.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigFile {
    /// Selected theme: can be a single name ("Nord") or table { light = "...", dark = "..." }
    pub theme: Option<ThemeSetting>,
    /// Legacy/simple preset field
    pub preset: Option<String>,
    /// Explicit light theme name
    pub light_theme: Option<String>,
    pub theme_light: Option<String>,
    /// Explicit dark theme name
    pub dark_theme: Option<String>,
    pub theme_dark: Option<String>,
    /// Optional inline palette definition
    pub palette: Option<PaletteConfig>,
    /// Optional diff color overrides
    pub diff: Option<DiffConfig>,
    pub diff_light: Option<DiffConfig>,
    pub diff_dark: Option<DiffConfig>,
    /// Optional line wrapping setting (defaults to true)
    pub wrap_lines: Option<bool>,
    /// Optional font size for diff content and line numbers (defaults to 13.0)
    pub font_size: Option<f32>,
    /// Installed font family for diff text and line numbers; defaults to monospace.
    pub font_family: Option<String>,
    pub whitespace: Option<crate::whitespace::WhitespaceMode>,
    pub tab_width: Option<usize>,
}

impl ConfigFile {
    pub fn get_theme_name(&self, is_dark: bool) -> String {
        let theme_pair_val = match &self.theme {
            Some(ThemeSetting::Pair { dark, light }) => {
                if is_dark {
                    dark.as_deref()
                } else {
                    light.as_deref()
                }
            }
            Some(ThemeSetting::Name(name)) => Some(name.as_str()),
            None => None,
        };

        if is_dark {
            self.dark_theme
                .as_deref()
                .or(self.theme_dark.as_deref())
                .or(theme_pair_val)
                .or(self.preset.as_deref())
                .unwrap_or("Dark")
                .trim()
                .to_string()
        } else {
            self.light_theme
                .as_deref()
                .or(self.theme_light.as_deref())
                .or(theme_pair_val)
                .or(self.preset.as_deref())
                .unwrap_or("Light")
                .trim()
                .to_string()
        }
    }
}

/// Structure of a user-defined theme file (e.g. `<name>.toml`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThemeFile {
    pub name: Option<String>,
    pub based_on: Option<String>,
    pub dark: Option<bool>,
    pub palette: Option<PaletteConfig>,
    pub diff: Option<DiffConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PaletteConfig {
    #[serde(default, deserialize_with = "deserialize_color")]
    pub background: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub text: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub primary: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub success: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub danger: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiffConfig {
    #[serde(default, deserialize_with = "deserialize_color")]
    pub delete_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub delete_highlight: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub delete_line_num: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub insert_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub insert_highlight: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub insert_line_num: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub equal_line_num: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub text: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub background: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub subheader_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub divider: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub empty_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub empty_cell_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub button_bg: Option<String>,
    #[serde(default, deserialize_with = "deserialize_color")]
    pub button_border: Option<String>,
}

fn deserialize_color<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let color = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = &color {
        parse_hex_color(value).map_err(serde::de::Error::custom)?;
    }
    Ok(color)
}

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("Could not read {}: {err}", path.display()))?;
    toml::from_str(&content)
        .map_err(|err| format!("Invalid configuration in {}: {err}", path.display()))
}

pub fn parse_hex_color(hex: &str) -> Result<Color, String> {
    let s = hex.trim().trim_start_matches('#');
    if !s.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("Invalid hex color: {hex:?}"));
    }
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).map_err(|e| e.to_string())?;
            Ok(Color::from_rgb8(r, g, b))
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&s[3..4].repeat(2), 16).map_err(|e| e.to_string())?;
            Ok(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
            Ok(Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&s[6..8], 16).map_err(|e| e.to_string())?;
            Ok(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        _ => Err(format!("Invalid hex color format: \"{}\"", hex)),
    }
}

/// Blends an overlay/foreground color `fg` with opacity `alpha` (0.0 to 1.0) over a background color `bg`.
/// Result = fg * alpha + bg * (1.0 - alpha).
pub fn blend(fg: Color, bg: Color, alpha: f32) -> Color {
    Color {
        r: fg.r * alpha + bg.r * (1.0 - alpha),
        g: fg.g * alpha + bg.g * (1.0 - alpha),
        b: fg.b * alpha + bg.b * (1.0 - alpha),
        a: 1.0,
    }
}

pub fn parse_preset(name: &str) -> Option<Theme> {
    match name.to_lowercase().replace(['-', '_', ' '], "").as_str() {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        "dracula" => Some(Theme::Dracula),
        "nord" => Some(Theme::Nord),
        "solarizedlight" => Some(Theme::SolarizedLight),
        "solarizeddark" => Some(Theme::SolarizedDark),
        "gruvboxlight" => Some(Theme::GruvboxLight),
        "gruvboxdark" => Some(Theme::GruvboxDark),
        "catppuccinlatte" => Some(Theme::CatppuccinLatte),
        "catppuccinfrappe" => Some(Theme::CatppuccinFrappe),
        "catppuccinmacchiato" => Some(Theme::CatppuccinMacchiato),
        "catppuccinmocha" => Some(Theme::CatppuccinMocha),
        "tokyonight" => Some(Theme::TokyoNight),
        "tokyonightstorm" => Some(Theme::TokyoNightStorm),
        "tokyonightlight" => Some(Theme::TokyoNightLight),
        "kanagawawave" => Some(Theme::KanagawaWave),
        "kanagawadragon" => Some(Theme::KanagawaDragon),
        "kanagawalotus" => Some(Theme::KanagawaLotus),
        "moonfly" => Some(Theme::Moonfly),
        "nightfly" => Some(Theme::Nightfly),
        "oxocarbon" => Some(Theme::Oxocarbon),
        "ferra" => Some(Theme::Ferra),
        _ => None,
    }
}

/// Returns prioritized list of configuration directories to look for `config.toml` and theme files.
pub fn config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = std::env::var("COMPARE_CONFIG_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(dir) = std::env::var("DIFFITRUST_CONFIG_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    // Current working directory
    dirs.push(PathBuf::from("."));
    // Standard User Config directories
    if let Ok(home) = std::env::var("HOME") {
        let home_path = PathBuf::from(home);
        dirs.push(home_path.join(".config").join("compare"));
        dirs.push(home_path.join(".config").join("diffitrust"));
        dirs.push(
            home_path
                .join("Library")
                .join("Application Support")
                .join("Compare"),
        );
        dirs.push(
            home_path
                .join("Library")
                .join("Application Support")
                .join("DiffiTrust"),
        );
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        dirs.push(PathBuf::from(&appdata).join("Compare"));
        dirs.push(PathBuf::from(appdata).join("DiffiTrust"));
    }
    dirs
}

/// Searches for `config.toml` across candidate directories.
pub fn find_config_file() -> (PathBuf, Option<PathBuf>) {
    let candidate_dirs = config_dirs();
    for dir in &candidate_dirs {
        let config_path = dir.join("config.toml");
        if config_path.is_file() {
            return (dir.clone(), Some(config_path));
        }
    }
    // Default fallback directory (e.g. ~/.config/diffitrust)
    let default_dir = candidate_dirs
        .get(2)
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."));
    (default_dir, None)
}

fn find_file_in_dir(dir: &Path, target_stem: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    let norm_target = target_stem.to_lowercase().replace(['-', '_', ' '], "");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let norm_stem = stem.to_lowercase().replace(['-', '_', ' '], "");
                            if norm_stem == norm_target {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Searches for a custom theme file (e.g. `<theme_name>.toml`) in the active config directory or search paths.
pub fn find_theme_file(theme_name: &str, active_config_dir: &Path) -> Option<PathBuf> {
    let clean_name = theme_name.trim_end_matches(".toml");

    // 1. Check in active config directory and its themes/ subdirectory
    if let Some(f) = find_file_in_dir(active_config_dir, clean_name) {
        return Some(f);
    }
    let active_themes = active_config_dir.join("themes");
    if let Some(f) = find_file_in_dir(&active_themes, clean_name) {
        return Some(f);
    }

    // 2. Check candidate directories
    for dir in config_dirs() {
        if dir == active_config_dir {
            continue;
        }
        if let Some(f) = find_file_in_dir(&dir, clean_name) {
            return Some(f);
        }
        let themes_dir = dir.join("themes");
        if let Some(f) = find_file_in_dir(&themes_dir, clean_name) {
            return Some(f);
        }
    }

    None
}

/// Creates default diff colors tailored to the given iced Theme (light or dark).
pub fn default_diff_theme(theme: &Theme) -> DiffTheme {
    let palette = theme.palette();
    let ext = theme.extended_palette();
    let is_dark = ext.is_dark;

    if is_dark {
        let delete_bg = blend(palette.danger, palette.background, 0.18);
        let delete_highlight = blend(palette.danger, palette.background, 0.40);
        let insert_bg = blend(palette.success, palette.background, 0.18);
        let insert_highlight = blend(palette.success, palette.background, 0.40);

        DiffTheme {
            delete_bg,
            delete_highlight,
            delete_line_num: palette.danger,
            insert_bg,
            insert_highlight,
            insert_line_num: palette.success,
            equal_line_num: blend(palette.text, palette.background, 0.45),
            text: palette.text,
            background: palette.background,
            subheader_bg: ext.background.weak.color,
            divider: ext.background.strong.color,
            empty_bg: palette.background,
            empty_cell_bg: blend(palette.background, ext.background.weak.color, 0.5),
            button_bg: ext.background.weak.color,
            button_border: ext.background.strong.color,
        }
    } else {
        DiffTheme {
            delete_bg: Color::from_rgb8(255, 235, 233),
            delete_highlight: Color::from_rgb8(255, 175, 175),
            delete_line_num: Color::from_rgb8(180, 50, 50),
            insert_bg: Color::from_rgb8(230, 255, 236),
            insert_highlight: Color::from_rgb8(172, 242, 189),
            insert_line_num: Color::from_rgb8(30, 140, 50),
            equal_line_num: Color::from_rgb8(140, 145, 155),
            text: palette.text,
            background: palette.background,
            subheader_bg: Color::from_rgb8(246, 248, 250),
            divider: Color::from_rgb8(225, 228, 232),
            empty_bg: Color::WHITE,
            empty_cell_bg: Color::from_rgb8(248, 249, 250),
            button_bg: Color::from_rgb8(243, 244, 246),
            button_border: Color::from_rgb8(215, 218, 222),
        }
    }
}

pub fn apply_diff_overrides(mut diff: DiffTheme, overrides: &DiffConfig) -> DiffTheme {
    if let Some(c) = overrides
        .delete_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.delete_bg = c;
    }
    if let Some(c) = overrides
        .delete_highlight
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.delete_highlight = c;
    }
    if let Some(c) = overrides
        .delete_line_num
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.delete_line_num = c;
    }
    if let Some(c) = overrides
        .insert_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.insert_bg = c;
    }
    if let Some(c) = overrides
        .insert_highlight
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.insert_highlight = c;
    }
    if let Some(c) = overrides
        .insert_line_num
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.insert_line_num = c;
    }
    if let Some(c) = overrides
        .equal_line_num
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.equal_line_num = c;
    }
    if let Some(c) = overrides
        .text
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.text = c;
    }
    if let Some(c) = overrides
        .background
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.background = c;
        diff.empty_bg = c;
    }
    if let Some(c) = overrides
        .subheader_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.subheader_bg = c;
    }
    if let Some(c) = overrides
        .divider
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.divider = c;
    }
    if let Some(c) = overrides
        .empty_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.empty_bg = c;
    }
    if let Some(c) = overrides
        .empty_cell_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.empty_cell_bg = c;
    }
    if let Some(c) = overrides
        .button_bg
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.button_bg = c;
    }
    if let Some(c) = overrides
        .button_border
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
    {
        diff.button_border = c;
    }
    diff
}

fn build_custom_theme(name: String, base_theme: &Theme, palette_cfg: &PaletteConfig) -> Theme {
    let base_palette = base_theme.palette();
    let bg = palette_cfg
        .background
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
        .unwrap_or(base_palette.background);
    let txt = palette_cfg
        .text
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
        .unwrap_or(base_palette.text);
    let pri = palette_cfg
        .primary
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
        .unwrap_or(base_palette.primary);
    let suc = palette_cfg
        .success
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
        .unwrap_or(base_palette.success);
    let dan = palette_cfg
        .danger
        .as_deref()
        .and_then(|s| parse_hex_color(s).ok())
        .unwrap_or(base_palette.danger);

    let palette = iced::theme::Palette {
        background: bg,
        text: txt,
        primary: pri,
        success: suc,
        danger: dan,
    };
    Theme::custom(name, palette)
}

/// Poll on the subscription executor, emitting only actual changes. Sending an
/// unchanged value to App::update would rebuild the entire UI in Iced 0.13.
pub fn system_theme_changes() -> iced::Subscription<bool> {
    iced::Subscription::run(theme_change_stream)
}

fn theme_change_stream() -> impl futures::Stream<Item = bool> {
    use futures::StreamExt;
    use iced::advanced::subscription::into_recipes;

    // Reuse Iced's executor-specific timer without adding another async runtime.
    let timer = into_recipes(iced::time::every(std::time::Duration::from_millis(500)))
        .pop()
        .expect("the timer has one subscription recipe")
        .stream(futures::stream::empty().boxed());
    distinct_theme_changes(timer.map(|_| poll_system_theme()), poll_system_theme())
}

// Executor threads do not have AppKit's per-event autorelease pool.
fn poll_system_theme() -> bool {
    #[cfg(target_os = "macos")]
    {
        objc::rc::autoreleasepool(is_system_dark)
    }
    #[cfg(not(target_os = "macos"))]
    {
        is_system_dark()
    }
}

fn distinct_theme_changes(
    source: impl futures::Stream<Item = bool>,
    mut previous: bool,
) -> impl futures::Stream<Item = bool> {
    use futures::StreamExt;
    source.filter_map(move |current| {
        let changed = (previous != current).then_some(current);
        previous = current;
        futures::future::ready(changed)
    })
}

/// Returns true if the operating system is currently in Dark mode.
pub fn is_system_dark() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            use objc::{class, msg_send, sel, sel_impl};
            let defaults: *mut objc::runtime::Object =
                msg_send![class!(NSUserDefaults), standardUserDefaults];
            if !defaults.is_null() {
                let key: *mut objc::runtime::Object = msg_send![
                    class!(NSString),
                    stringWithUTF8String: c"AppleInterfaceStyle".as_ptr()
                ];
                if !key.is_null() {
                    let val: *mut objc::runtime::Object = msg_send![defaults, stringForKey: key];
                    if !val.is_null() {
                        let utf8: *const std::os::raw::c_char = msg_send![val, UTF8String];
                        if !utf8.is_null() {
                            if let Ok(s) = std::ffi::CStr::from_ptr(utf8).to_str() {
                                return s.eq_ignore_ascii_case("dark");
                            }
                        }
                    } else {
                        // Key does not exist -> macOS is in Light mode
                        return false;
                    }
                }
            }
        }
    }

    matches!(dark_light::detect(), dark_light::Mode::Dark)
}

pub fn resolve_theme(
    theme_query: &str,
    is_dark: bool,
    config: &ConfigFile,
    config_dir: &Path,
) -> AppTheme {
    let mut warning = None;
    // 1. Check if there is a user-defined theme file matching `theme_query`
    if let Some(theme_file_path) = find_theme_file(theme_query, config_dir) {
        match read_toml::<ThemeFile>(&theme_file_path) {
            Ok(theme_file) => {
                let base_theme = theme_file
                    .based_on
                    .as_deref()
                    .and_then(parse_preset)
                    .or_else(|| parse_preset(theme_query))
                    .unwrap_or(if is_dark { Theme::Dark } else { Theme::Light });

                let resolved_theme = if let Some(pal) = &theme_file.palette {
                    build_custom_theme(
                        theme_file.name.unwrap_or_else(|| theme_query.to_string()),
                        &base_theme,
                        pal,
                    )
                } else {
                    base_theme
                };

                let mut diff = default_diff_theme(&resolved_theme);
                if let Some(theme_diff) = &theme_file.diff {
                    diff = apply_diff_overrides(diff, theme_diff);
                }
                if let Some(cfg_diff) = &config.diff {
                    diff = apply_diff_overrides(diff, cfg_diff);
                }
                if is_dark {
                    if let Some(cfg_dark) = &config.diff_dark {
                        diff = apply_diff_overrides(diff, cfg_dark);
                    }
                } else if let Some(cfg_light) = &config.diff_light {
                    diff = apply_diff_overrides(diff, cfg_light);
                }

                let is_dark_resolved = resolved_theme.extended_palette().is_dark;
                return AppTheme {
                    name: theme_query.to_string(),
                    iced_theme: resolved_theme,
                    diff,
                    is_dark: is_dark_resolved,
                    warning,
                };
            }
            Err(err) => warning = Some(err),
        }
    }

    // 2. Check if `theme_query` matches a built-in preset
    let base_theme =
        parse_preset(theme_query).unwrap_or(if is_dark { Theme::Dark } else { Theme::Light });

    let resolved_theme = if let Some(pal) = &config.palette {
        build_custom_theme(format!("{}-custom", theme_query), &base_theme, pal)
    } else {
        base_theme
    };

    let mut diff = default_diff_theme(&resolved_theme);
    if let Some(cfg_diff) = &config.diff {
        diff = apply_diff_overrides(diff, cfg_diff);
    }
    if is_dark {
        if let Some(cfg_dark) = &config.diff_dark {
            diff = apply_diff_overrides(diff, cfg_dark);
        }
    } else if let Some(cfg_light) = &config.diff_light {
        diff = apply_diff_overrides(diff, cfg_light);
    }

    let is_dark_resolved = resolved_theme.extended_palette().is_dark;
    AppTheme {
        name: theme_query.to_string(),
        iced_theme: resolved_theme,
        diff,
        is_dark: is_dark_resolved,
        warning,
    }
}

/// Loads the application theme for the current operating system appearance (light or dark).
pub fn load_theme() -> AppTheme {
    let is_dark = is_system_dark();
    load_theme_for_mode(is_dark)
}

/// Read settings once and retain diagnostics for the application to display.
pub fn load_settings(is_dark: bool) -> (ConfigFile, AppTheme) {
    let (config_dir, config_path) = find_config_file();
    load_settings_from(&config_dir, config_path.as_deref(), is_dark)
}

fn load_settings_from(
    config_dir: &Path,
    config_path: Option<&Path>,
    is_dark: bool,
) -> (ConfigFile, AppTheme) {
    let result = config_path.map(read_toml::<ConfigFile>).transpose();
    let (config, warning) = match result {
        Ok(config) => (config.unwrap_or_default(), None),
        Err(err) => (ConfigFile::default(), Some(err)),
    };
    let mut theme = resolve_theme(
        &config.get_theme_name(is_dark),
        is_dark,
        &config,
        config_dir,
    );
    theme.warning = match (warning, theme.warning) {
        (Some(a), Some(b)) => Some(format!("{a}\n{b}")),
        (a, b) => a.or(b),
    };
    (config, theme)
}

/// Loads the application theme for an explicit mode (true for dark, false for light).
pub fn load_theme_for_mode(is_dark: bool) -> AppTheme {
    load_settings(is_dark).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_config_accepts_modes_and_tab_width() {
        use crate::whitespace::WhitespaceMode;
        for (name, mode) in [
            ("off", WhitespaceMode::Off),
            ("changed", WhitespaceMode::Changed),
            ("all", WhitespaceMode::All),
        ] {
            let config: ConfigFile =
                toml::from_str(&format!("whitespace = \"{name}\"\ntab_width = 8")).unwrap();
            assert_eq!(config.whitespace, Some(mode));
            assert_eq!(config.tab_width, Some(8));
        }
        assert!(toml::from_str::<ConfigFile>("whitespace = \"typo\"").is_err());
        assert!(toml::from_str::<ConfigFile>("tab_width = -1").is_err());
    }

    #[test]
    fn malformed_colors_return_errors_without_panicking() {
        for color in ["€", "éa", "💥", "#€€", "#éabcdef", "#ggg", "#12"] {
            assert!(parse_hex_color(color).is_err(), "{color}");
        }
    }

    #[test]
    fn invalid_colors_report_the_config_key() {
        for section in ["diff", "diff_light", "diff_dark"] {
            let config = format!("[{section}]\ndelete_bg = \"€\"\n");
            let error = toml::from_str::<ConfigFile>(&config)
                .unwrap_err()
                .to_string();
            assert!(error.contains("delete_bg"));
            assert!(error.contains("Invalid hex color"));
        }
        assert!(toml::from_str::<ThemeFile>("[palette]\nbackground = \"oops\"").is_err());
    }

    #[test]
    fn configuration_failures_include_path_and_use_defaults() {
        let directory =
            std::env::temp_dir().join(format!("compare_diagnostics_{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("config.toml");
        for content in ["font_size = [", "[diff]\ndelete_bg = \"€\""] {
            fs::write(&path, content).unwrap();
            let (config, theme) = load_settings_from(&directory, Some(&path), true);
            assert!(config.font_size.is_none());
            assert_eq!(theme.name, "Dark");
            let warning = theme.warning.unwrap();
            assert!(warning.contains(path.to_str().unwrap()));
            assert!(warning.contains("Invalid configuration"));
        }
        let missing = directory.join("missing.toml");
        let (_, theme) = load_settings_from(&directory, Some(&missing), false);
        assert!(theme.warning.unwrap().contains("Could not read"));
        let (_, theme) = load_settings_from(&directory, None, false);
        assert!(theme.warning.is_none());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_theme_file_reports_error_and_falls_back() {
        let directory =
            std::env::temp_dir().join(format!("compare_theme_diagnostics_{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("Nord.toml");
        fs::write(&path, "[diff]\ndelete_bg = \"€\"").unwrap();
        let theme = resolve_theme("Nord", true, &ConfigFile::default(), &directory);
        assert_eq!(theme.iced_theme, Theme::Nord);
        assert!(theme.warning.unwrap().contains(path.to_str().unwrap()));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#fff").unwrap(), Color::WHITE);
        assert_eq!(parse_hex_color("ffffff").unwrap(), Color::WHITE);
        assert_eq!(parse_hex_color("#000000").unwrap(), Color::BLACK);

        let red = parse_hex_color("#ff0000").unwrap();
        assert_eq!(red, Color::from_rgb(1.0, 0.0, 0.0));

        let with_alpha = parse_hex_color("#ff000080").unwrap();
        assert!((with_alpha.a - 0.5019).abs() < 0.01);

        assert!(parse_hex_color("invalid").is_err());
    }

    #[test]
    fn test_parse_presets() {
        assert_eq!(
            parse_preset("CatppuccinMocha"),
            Some(Theme::CatppuccinMocha)
        );
        assert_eq!(
            parse_preset("catppuccin-mocha"),
            Some(Theme::CatppuccinMocha)
        );
        assert_eq!(parse_preset("dracula"), Some(Theme::Dracula));
        assert_eq!(parse_preset("Nord"), Some(Theme::Nord));
        assert_eq!(parse_preset("solarized-dark"), Some(Theme::SolarizedDark));
        assert_eq!(parse_preset("UnknownTheme"), None);
    }

    #[test]
    fn test_apply_diff_overrides() {
        let default_diff = default_diff_theme(&Theme::Light);
        let overrides = DiffConfig {
            delete_bg: Some("#112233".to_string()),
            insert_bg: Some("#445566".to_string()),
            empty_cell_bg: Some("#778899".to_string()),
            ..Default::default()
        };
        let customized = apply_diff_overrides(default_diff, &overrides);
        assert_eq!(customized.delete_bg, parse_hex_color("#112233").unwrap());
        assert_eq!(customized.insert_bg, parse_hex_color("#445566").unwrap());
        assert_eq!(
            customized.empty_cell_bg,
            parse_hex_color("#778899").unwrap()
        );
    }

    #[test]
    fn test_config_toml_deserialization() {
        let toml_str = r##"
            theme = "CatppuccinMocha"

            [diff]
            delete_bg = "#3a1520"
            insert_bg = "#183226"
            insert_line_num = "#38bdf8"
        "##;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.theme,
            Some(ThemeSetting::Name("CatppuccinMocha".to_string()))
        );
        assert_eq!(config.get_theme_name(true), "CatppuccinMocha");
        assert_eq!(config.get_theme_name(false), "CatppuccinMocha");
        let diff = config.diff.unwrap();
        assert_eq!(diff.delete_bg.as_deref(), Some("#3a1520"));
        assert_eq!(diff.insert_line_num.as_deref(), Some("#38bdf8"));
    }

    #[test]
    fn test_find_theme_file_case_insensitive() {
        let temp_dir = std::env::temp_dir().join(format!("diffitrust_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let theme_file_path = temp_dir.join("Nord.toml");
        let _ = fs::write(&theme_file_path, "name = \"Nord\"\n");

        let found = find_theme_file("nord", &temp_dir);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), theme_file_path);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_theme_file_override_preset() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("diffitrust_override_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let config_path = temp_dir.join("config.toml");
        let _ = fs::write(
            &config_path,
            r#"
            preset = "CatppuccinMocha"
            "#,
        );

        let theme_file_path = temp_dir.join("CatppuccinMocha.toml");
        let _ = fs::write(
            &theme_file_path,
            r##"
            based_on = "CatppuccinMocha"

            [diff]
            delete_bg = "#432029"
            insert_bg = "#1e3a2f"
            "##,
        );

        std::env::set_var("DIFFITRUST_CONFIG_DIR", &temp_dir);
        let app_theme = load_theme();
        std::env::remove_var("DIFFITRUST_CONFIG_DIR");

        assert_eq!(app_theme.name, "CatppuccinMocha");
        assert!(app_theme.is_dark);
        assert_eq!(
            app_theme.diff.delete_bg,
            parse_hex_color("#432029").unwrap()
        );
        assert_eq!(
            app_theme.diff.insert_bg,
            parse_hex_color("#1e3a2f").unwrap()
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_custom_theme_file_loading() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("diffitrust_custom_test_{}", std::process::id()));
        let themes_dir = temp_dir.join("themes");
        let _ = fs::create_dir_all(&themes_dir);

        let config_path = temp_dir.join("config.toml");
        let _ = fs::write(
            &config_path,
            r#"
            theme = "cyan-diff"
            "#,
        );

        let theme_file_path = themes_dir.join("cyan-diff.toml");
        let _ = fs::write(
            &theme_file_path,
            r##"
            name = "Cyan Diff"
            based_on = "Nord"

            [diff]
            delete_bg = "#3b222e"
            insert_bg = "#1f3a3a"
            insert_line_num = "#88c0d0"
            "##,
        );

        std::env::set_var("DIFFITRUST_CONFIG_DIR", &temp_dir);
        let app_theme = load_theme();
        std::env::remove_var("DIFFITRUST_CONFIG_DIR");

        assert_eq!(app_theme.name, "cyan-diff");
        assert!(app_theme.is_dark);
        assert_eq!(
            app_theme.diff.delete_bg,
            parse_hex_color("#3b222e").unwrap()
        );
        assert_eq!(
            app_theme.diff.insert_bg,
            parse_hex_color("#1f3a3a").unwrap()
        );
        assert_eq!(
            app_theme.diff.insert_line_num,
            parse_hex_color("#88c0d0").unwrap()
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_blend() {
        let black = Color::BLACK;
        let white = Color::WHITE;

        // 10% black over white should be light gray (0.9, 0.9, 0.9)
        let blended = blend(black, white, 0.10);
        assert!((blended.r - 0.9).abs() < 1e-4);
        assert!((blended.g - 0.9).abs() < 1e-4);
        assert!((blended.b - 0.9).abs() < 1e-4);

        // 10% white over black should be dark gray (0.1, 0.1, 0.1)
        let blended_dark = blend(white, black, 0.10);
        assert!((blended_dark.r - 0.1).abs() < 1e-4);
        assert!((blended_dark.g - 0.1).abs() < 1e-4);
        assert!((blended_dark.b - 0.1).abs() < 1e-4);
    }

    #[test]
    fn test_light_dark_theme_options_in_config_toml() {
        // Syntax 1: light_theme and dark_theme top-level keys
        let toml_str1 = r#"
            light_theme = "kaleidescope"
            dark_theme = "Nord"
        "#;
        let config1: ConfigFile = toml::from_str(toml_str1).unwrap();
        assert_eq!(config1.get_theme_name(false), "kaleidescope");
        assert_eq!(config1.get_theme_name(true), "Nord");

        // Syntax 2: [theme] section with light and dark keys
        let toml_str2 = r#"
            [theme]
            light = "CatppuccinLatte"
            dark = "CatppuccinMocha"
        "#;
        let config2: ConfigFile = toml::from_str(toml_str2).unwrap();
        assert_eq!(config2.get_theme_name(false), "CatppuccinLatte");
        assert_eq!(config2.get_theme_name(true), "CatppuccinMocha");

        // Syntax 3: single preset fallback
        let toml_str3 = r#"
            preset = "SolarizedLight"
        "#;
        let config3: ConfigFile = toml::from_str(toml_str3).unwrap();
        assert_eq!(config3.get_theme_name(false), "SolarizedLight");
        assert_eq!(config3.get_theme_name(true), "SolarizedLight");
    }

    #[test]
    fn test_load_theme_for_mode() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir =
            std::env::temp_dir().join(format!("diffitrust_mode_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let config_path = temp_dir.join("config.toml");
        let _ = fs::write(
            &config_path,
            r#"
            light_theme = "Light"
            dark_theme = "Nord"
            "#,
        );

        std::env::set_var("DIFFITRUST_CONFIG_DIR", &temp_dir);
        let light_app_theme = load_theme_for_mode(false);
        let dark_app_theme = load_theme_for_mode(true);
        std::env::remove_var("DIFFITRUST_CONFIG_DIR");

        assert_eq!(light_app_theme.name, "Light");
        assert!(!light_app_theme.is_dark);

        assert_eq!(dark_app_theme.name, "Nord");
        assert!(dark_app_theme.is_dark);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_is_system_dark_returns_bool() {
        let _is_dark = is_system_dark();
    }

    #[test]
    fn test_wrap_lines_config() {
        let toml_str = r#"
            wrap_lines = false
        "#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.wrap_lines, Some(false));
    }

    #[test]
    fn test_font_size_config() {
        let toml_str = r#"
            font_size = 15.5
        "#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.font_size, Some(15.5));

        let toml_int = r#"
            font_size = 14
        "#;
        let config_int: ConfigFile = toml::from_str(toml_int).unwrap();
        assert_eq!(config_int.font_size, Some(14.0));
    }

    #[test]
    fn theme_subscription_emits_changes_without_periodic_noop_messages() {
        use futures::{executor::block_on, stream, StreamExt};
        let values = [false, false, true, true, true, false, false, true];
        let changes =
            block_on(distinct_theme_changes(stream::iter(values), false).collect::<Vec<_>>());
        assert_eq!(changes, vec![true, false, true]);
        let unchanged =
            block_on(distinct_theme_changes(stream::iter([true; 100]), true).collect::<Vec<_>>());
        assert!(unchanged.is_empty());
    }
}
