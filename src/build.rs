//! Slow, local steps: fetch originals, extract Aether palettes, draw previews.
//! These need the images and the `aether` CLI, so CI does not run them.

use crate::manifest::{self, Row};
use crate::model::{Blueprint, Palette, Theme, blueprint_path, preview_path, read_blueprint, write_json};
use anyhow::{Context, Result, bail, ensure};
use image::imageops::{self, FilterType};
use image::{DynamicImage, Rgb, RgbImage};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const PREVIEW: (u32, u32) = (640, 360);
const MOSAIC_TILE: (u32, u32) = (400, 225);
const MOSAIC_COLUMNS: u32 = 4;
const MOSAIC_GAP: u32 = 12;
const HERO_COLUMNS: u32 = 6;

pub struct Options {
    pub caches: Vec<PathBuf>,
    pub force: bool,
}

/// The image cache the installer uses, followed by any extra caches.
pub fn default_caches() -> Vec<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let cache = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".cache"));
    vec![cache.join("omarchy-wallpapers")]
}

pub fn run(root: &Path, options: &Options) -> Result<()> {
    let rows = manifest::read(&root.join("wallpapers.tsv"))?;
    manifest::validate(&rows)?;
    let themes = read_themes(&rows)?;
    write_json(&root.join("themes.json"), &themes)?;

    // Blueprints and previews need the original, so fetch what is stale first.
    let stale: Vec<&Row> = rows
        .iter()
        .filter(|r| {
            options.force
                || !root.join(preview_path(r)).exists()
                || read_blueprint(root, r).map_or(true, |b| b.wallpaper_sha256 != r.sha256 || b.palette.wallpaper_url != r.url())
        })
        .collect();
    eprintln!("{} of {} wallpapers need a palette or preview.", stale.len(), rows.len());
    for (index, r) in stale.iter().enumerate() {
        let original = original(root, r, &options.caches)?;
        let theme = &themes[&r.theme];
        let colors = extract_palette(&original, theme.light())?;
        let blueprint = Blueprint {
            name: format!("{} - {}", theme.title, r.title()),
            palette: Palette {
                colors,
                wallpaper_url: r.url(),
                light_mode: theme.light(),
                mode: theme.mode.clone(),
            },
            timestamp: 0,
            wallpaper_sha256: r.sha256.clone(),
        };
        write_json(&root.join(blueprint_path(r)), &blueprint)?;
        write_preview(&original, &root.join(preview_path(r)))?;
        eprintln!("[{:>3}/{}] {}/{}", index + 1, stale.len(), r.theme, r.slug());
    }

    let groups = manifest::by_theme(&rows);
    groups.par_iter().try_for_each(|(theme, rows)| write_mosaic(root, &themes[*theme], rows))?;
    write_hero(root, &groups)?;
    Ok(())
}

/// Each theme's mode and colours from its Omarchy `colors.toml`.
fn read_themes(rows: &[Row]) -> Result<BTreeMap<String, Theme>> {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let system = std::env::var_os("OMARCHY_PATH").map(PathBuf::from).unwrap_or_else(|| "/usr/share/omarchy".into());
    let dirs = [home.join(".config/omarchy/themes"), system.join("themes")];
    let mut themes = BTreeMap::new();
    for name in manifest::by_theme(rows).keys() {
        let Some(path) = dirs.iter().map(|d| d.join(name).join("colors.toml")).find(|p| p.exists()) else {
            bail!("theme {name} is not installed; install it so its colours can be read");
        };
        let text = std::fs::read_to_string(&path)?;
        let value = |key: &str| -> Option<String> {
            text.lines().find_map(|l| {
                let (k, v) = l.split_once('=')?;
                (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
            })
        };
        let background = value("background").with_context(|| format!("{}: no background", path.display()))?;
        let mode = value("mode").unwrap_or_else(|| if path.with_file_name("light.mode").exists() { "light" } else { "dark" }.into());
        themes.insert(
            name.to_string(),
            Theme {
                title: name.split('-').map(capitalize).collect::<Vec<_>>().join(" "),
                mode,
                foreground: value("foreground").unwrap_or_else(|| "#ffffff".into()),
                accent: value("accent").unwrap_or_else(|| "#888888".into()),
                background,
            },
        );
    }
    Ok(themes)
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

fn sha256(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut std::fs::File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Find the original by hash in the repository or a cache; download it if missing.
fn original(root: &Path, row: &Row, caches: &[PathBuf]) -> Result<PathBuf> {
    if row.local() {
        let path = root.join(&row.source);
        ensure!(sha256(&path)? == row.sha256, "{}: checksum mismatch", path.display());
        return Ok(path);
    }
    for cache in caches {
        let path = cache.join(&row.sha256);
        if path.exists() && sha256(&path)? == row.sha256 {
            return Ok(path);
        }
    }
    let cache = &caches[0];
    std::fs::create_dir_all(cache)?;
    let partial = cache.join(format!(".download.{}", row.sha256));
    // Wikimedia throttles bursts, so space out downloads.
    std::thread::sleep(Duration::from_secs(2));
    let status = Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location", "--retry", "3", "--output"])
        .arg(&partial)
        .arg(row.url())
        .status()
        .context("run curl")?;
    ensure!(status.success(), "download failed: {}", row.url());
    ensure!(sha256(&partial)? == row.sha256, "{}: checksum mismatch for {}", row.file, row.url());
    let path = cache.join(&row.sha256);
    std::fs::rename(&partial, &path)?;
    Ok(path)
}

#[derive(serde::Deserialize)]
struct Extracted {
    colors: Vec<String>,
}

/// The 16 colours Aether picks for this image, the same palette `aether --generate` applies.
fn extract_palette(path: &Path, light: bool) -> Result<Vec<String>> {
    let mut command = Command::new("aether");
    command.arg("--extract-palette").arg(path).arg("--json");
    if light {
        command.arg("--light-mode");
    }
    let output = command.output().context("run aether; install it with `omarchy pkg add aether` or from github.com/omacom/aether")?;
    ensure!(output.status.success(), "aether failed on {}: {}", path.display(), String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json = &stdout[stdout.find('{').context("aether printed no JSON")?..];
    let extracted: Extracted = serde_json::from_str(json)?;
    ensure!(extracted.colors.len() == 16, "aether gave {} colours", extracted.colors.len());
    Ok(extracted.colors.into_iter().map(|c| c.to_lowercase()).collect())
}

fn write_jpeg(image: &RgbImage, path: &Path, quality: u8) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    let file = std::fs::File::create(path)?;
    image::codecs::jpeg::JpegEncoder::new_with_quality(BufWriter::new(file), quality)
        .encode_image(image)
        .with_context(|| format!("encode {}", path.display()))
}

fn write_preview(original: &Path, out: &Path) -> Result<()> {
    let image = image::ImageReader::open(original)?
        .with_guessed_format()?
        .decode()
        .with_context(|| format!("decode {}", original.display()))?;
    let preview = image.resize_to_fill(PREVIEW.0, PREVIEW.1, FilterType::Lanczos3).to_rgb8();
    write_jpeg(&preview, out, 84)
}

fn hex(color: &str) -> Rgb<u8> {
    let v = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0);
    Rgb([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn tile(root: &Path, row: &Row, size: (u32, u32)) -> Result<RgbImage> {
    let path = root.join(preview_path(row));
    let preview = image::open(&path).with_context(|| format!("open {}", path.display()))?;
    Ok(DynamicImage::from(preview).resize_to_fill(size.0, size.1, FilterType::Lanczos3).to_rgb8())
}

/// A contact sheet per theme, laid out on the theme's own background colour.
fn write_mosaic(root: &Path, theme: &Theme, rows: &[&Row]) -> Result<()> {
    let (w, h) = MOSAIC_TILE;
    let count = rows.len() as u32;
    let lines = count.div_ceil(MOSAIC_COLUMNS);
    let pad = MOSAIC_GAP * 2;
    let width = pad * 2 + MOSAIC_COLUMNS * w + (MOSAIC_COLUMNS - 1) * MOSAIC_GAP;
    let height = pad * 2 + lines * h + (lines - 1) * MOSAIC_GAP;
    let mut sheet = RgbImage::from_pixel(width, height, hex(&theme.background));
    for (i, row) in rows.iter().enumerate() {
        let (col, line) = (i as u32 % MOSAIC_COLUMNS, i as u32 / MOSAIC_COLUMNS);
        let x = pad + col * (w + MOSAIC_GAP);
        let y = pad + line * (h + MOSAIC_GAP);
        imageops::overlay(&mut sheet, &tile(root, row, MOSAIC_TILE)?, x.into(), y.into());
    }
    write_jpeg(&sheet, &root.join(format!("docs/mosaics/{}.jpg", rows[0].theme)), 82)
}

/// One wallpaper per theme, edge to edge: an image stored in this repository
/// if the theme has one, else the first that no earlier theme already shows.
fn write_hero(root: &Path, groups: &BTreeMap<&str, Vec<&Row>>) -> Result<()> {
    let (w, h) = MOSAIC_TILE;
    let lines = (groups.len() as u32).div_ceil(HERO_COLUMNS);
    let mut hero = RgbImage::from_pixel(HERO_COLUMNS * w, lines * h, Rgb([12, 12, 12]));
    let mut used = std::collections::HashSet::new();
    for (i, rows) in groups.values().enumerate() {
        let (col, line) = (i as u32 % HERO_COLUMNS, i as u32 / HERO_COLUMNS);
        let fresh = |r: &&&Row| !used.contains(&r.sha256);
        let row = rows.iter().filter(|r| r.local()).find(fresh).or_else(|| rows.iter().find(fresh)).unwrap_or(&rows[0]);
        used.insert(row.sha256.clone());
        imageops::overlay(&mut hero, &tile(root, row, MOSAIC_TILE)?, (col * w).into(), (line * h).into());
    }
    write_jpeg(&hero, &root.join("docs/hero.jpg"), 84)
}
