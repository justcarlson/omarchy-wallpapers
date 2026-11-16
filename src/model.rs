use crate::manifest::Row;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Colours from a theme's Omarchy `colors.toml`, kept in `themes.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub title: String,
    pub mode: String,
    pub background: String,
    pub foreground: String,
    pub accent: String,
}

impl Theme {
    pub fn light(&self) -> bool {
        self.mode == "light"
    }
}

/// An Aether blueprint. Aether reads `name` and `palette`; it ignores
/// `wallpaperSha256`, which records the image the palette came from.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blueprint {
    pub name: String,
    pub palette: Palette,
    pub timestamp: i64,
    pub wallpaper_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub colors: Vec<String>,
    pub wallpaper_url: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub light_mode: bool,
    pub mode: String,
}

pub fn blueprint_path(row: &Row) -> PathBuf {
    PathBuf::from(format!("blueprints/{}/{}.json", row.theme, row.slug()))
}

pub fn preview_path(row: &Row) -> PathBuf {
    PathBuf::from(format!("docs/previews/{}/{}.jpg", row.theme, row.slug()))
}

pub fn read_blueprint(root: &Path, row: &Row) -> Result<Blueprint> {
    let path = root.join(blueprint_path(row));
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}
