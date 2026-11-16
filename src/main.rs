//! `gallery`: keeps the wallpaper collection, its previews, Aether blueprints
//! and pages in step with `wallpapers.tsv`.

mod build;
mod manifest;
mod model;
mod render;

use anyhow::{Context, Result, bail, ensure};
use manifest::Row;
use model::{blueprint_path, preview_path, read_blueprint};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
Usage: gallery <command>

  sync <dir> [--cache <dir>]...
                    Copy a curator's manifest (themes.tsv, optional dracula.tsv, images/) into this repository
  build [--force] [--cache <dir>]...
                    Fetch originals, extract Aether palettes, draw previews and mosaics, then render
  render            Rebuild README.md, themes/*.md and docs/index.html from the committed data
  check             Verify the manifest, blueprints, previews and pages agree (CI runs this)
";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = std::env::current_dir()?;
    ensure!(root.join("wallpapers.tsv").exists() || args.first().map(String::as_str) == Some("sync"), "run gallery from the repository root");
    match args.first().map(String::as_str) {
        Some("sync") => {
            let mut caches = build::default_caches();
            let mut dir = None;
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.as_str() {
                    "--cache" => caches.push(rest.next().context("--cache needs a directory")?.into()),
                    _ if dir.is_none() && !arg.starts_with('-') => dir = Some(PathBuf::from(arg)),
                    _ => bail!(USAGE),
                }
            }
            sync(&root, &dir.context(USAGE)?, &caches)
        }
        Some("build") => {
            let mut options = build::Options { caches: build::default_caches(), force: false };
            let mut rest = args[1..].iter();
            while let Some(arg) = rest.next() {
                match arg.as_str() {
                    "--force" => options.force = true,
                    "--cache" => options.caches.push(rest.next().context("--cache needs a directory")?.into()),
                    _ => bail!(USAGE),
                }
            }
            build::run(&root, &options)?;
            prune(&root)?;
            write_outputs(&root)
        }
        Some("render") => write_outputs(&root),
        Some("check") => check(&root),
        _ => {
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    }
}

fn sha256(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Full-size Wikimedia Commons files. Commons answers repeated downloads of
/// these with HTTP 429 and asks clients to use thumbnails, so a painting with
/// no large enough thumbnail is stored in `images/` instead.
fn commons_original(source: &str) -> bool {
    source.starts_with("https://upload.wikimedia.org/") && !source.contains("/thumb/")
}

/// Import the curator's manifests. Dracula rows use the older three-column form.
fn sync(root: &Path, from: &Path, caches: &[PathBuf]) -> Result<()> {
    let mut rows = manifest::read(&from.join("themes.tsv"))?;
    let dracula = from.join("dracula.tsv");
    if dracula.exists() {
        for line in std::fs::read_to_string(&dracula)?.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let fields: Vec<&str> = line.split('\t').collect();
            let [sha256, file, source] = fields[..] else { bail!("dracula.tsv: want 3 fields: {line}") };
            if file.ends_with(".txt") {
                continue; // Licence files; CREDITS.md links them.
            }
            rows.push(Row { theme: "dracula".into(), sha256: sha256.into(), file: format!("curated-{file}"), source: source.into(), origin: None });
        }
    }
    manifest::validate(&rows)?;

    std::fs::create_dir_all(root.join("images"))?;
    for r in rows.iter().filter(|r| r.local()) {
        std::fs::copy(from.join(&r.source), root.join(&r.source)).with_context(|| format!("copy {}", r.source))?;
    }
    for r in rows.iter_mut().filter(|r| commons_original(&r.source)) {
        let stored = format!("images/{}", r.file.strip_prefix("curated-").unwrap_or(&r.file));
        let target = root.join(&stored);
        if !(target.exists() && sha256(&target)? == r.sha256) {
            std::fs::copy(build::fetch(&r.source, &r.sha256, caches)?, &target).with_context(|| format!("store {stored}"))?;
        }
        r.origin = Some(std::mem::replace(&mut r.source, stored));
    }
    manifest::validate(&rows)?;

    let wanted: BTreeSet<&str> = rows.iter().filter(|r| r.local()).map(|r| r.source.as_str()).collect();
    if let Ok(entries) = std::fs::read_dir(root.join("images")) {
        for entry in entries {
            let path = entry?.path();
            let relative = format!("images/{}", path.file_name().unwrap().to_string_lossy());
            if !wanted.contains(relative.as_str()) && !relative.ends_with(".md") {
                std::fs::remove_file(&path)?;
                eprintln!("removed {relative}");
            }
        }
    }

    let before = manifest::read(&root.join("wallpapers.tsv")).unwrap_or_default();
    let key = |r: &Row| format!("{}/{}/{}", r.theme, r.file, r.sha256);
    let old: BTreeSet<String> = before.iter().map(key).collect();
    let new: BTreeSet<String> = rows.iter().map(key).collect();
    manifest::write(&root.join("wallpapers.tsv"), &rows)?;
    eprintln!(
        "Synced {} wallpapers: {} added or changed, {} removed. Next: gallery build",
        rows.len(),
        new.difference(&old).count(),
        old.difference(&new).count()
    );
    Ok(())
}

/// Generated files that the manifest no longer names.
fn orphans(root: &Path, rows: &[Row]) -> Result<Vec<PathBuf>> {
    let mut wanted: BTreeSet<PathBuf> = BTreeSet::new();
    for r in rows {
        wanted.insert(blueprint_path(r));
        wanted.insert(preview_path(r));
    }
    for theme in manifest::by_theme(rows).keys() {
        wanted.insert(format!("docs/mosaics/{theme}.jpg").into());
        wanted.insert(format!("themes/{theme}.md").into());
    }
    let mut found = Vec::new();
    for dir in ["blueprints", "docs/previews", "docs/mosaics", "themes"] {
        walk(root, &root.join(dir), &mut found)?;
    }
    Ok(found.into_iter().filter(|p| !wanted.contains(p)).collect())
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Ok(()) };
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else {
            out.push(path.strip_prefix(root)?.to_path_buf());
        }
    }
    Ok(())
}

fn prune(root: &Path) -> Result<()> {
    let rows = manifest::read(&root.join("wallpapers.tsv"))?;
    for path in orphans(root, &rows)? {
        std::fs::remove_file(root.join(&path))?;
        eprintln!("removed {}", path.display());
    }
    Ok(())
}

fn write_outputs(root: &Path) -> Result<()> {
    let inputs = render::load(root)?;
    for (path, text) in render::outputs(root, &inputs)? {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap())?;
        if std::fs::read_to_string(&path).ok().as_deref() != Some(&text) {
            std::fs::write(&path, text)?;
            eprintln!("wrote {}", path.strip_prefix(root)?.display());
        }
    }
    Ok(())
}

fn check(root: &Path) -> Result<()> {
    let inputs = render::load(root)?;
    let mut problems = Vec::new();
    for r in &inputs.rows {
        let at = format!("{}/{}", r.theme, r.file);
        if r.local() && sha256(&root.join(&r.source)).ok().as_deref() != Some(r.sha256.as_str()) {
            problems.push(format!("{at}: {} is missing or does not match its SHA-256", r.source));
        }
        let blueprint = read_blueprint(root, r)?;
        if blueprint.wallpaper_sha256 != r.sha256 || blueprint.palette.wallpaper_url != r.url() {
            problems.push(format!("{at}: blueprint is stale; run gallery build"));
        }
        let colors = &blueprint.palette.colors;
        if colors.len() != 16 || !colors.iter().all(|c| c.len() == 7 && c.starts_with('#') && c[1..].bytes().all(|b| b.is_ascii_hexdigit())) {
            problems.push(format!("{at}: blueprint palette must be 16 #rrggbb colours"));
        }
        if blueprint.palette.light_mode != inputs.themes[&r.theme].light() {
            problems.push(format!("{at}: blueprint mode does not match the theme"));
        }
        if !root.join(preview_path(r)).exists() {
            problems.push(format!("{at}: no preview; run gallery build"));
        }
    }
    for theme in manifest::by_theme(&inputs.rows).keys() {
        if !root.join(format!("docs/mosaics/{theme}.jpg")).exists() {
            problems.push(format!("{theme}: no mosaic; run gallery build"));
        }
    }
    for path in orphans(root, &inputs.rows)? {
        problems.push(format!("{}: not in the manifest", path.display()));
    }
    for (path, text) in render::outputs(root, &inputs)? {
        if std::fs::read_to_string(root.join(&path)).ok().as_deref() != Some(text.as_str()) {
            problems.push(format!("{}: out of date; run gallery render", path.display()));
        }
    }
    if !problems.is_empty() {
        for p in &problems {
            eprintln!("{p}");
        }
        bail!("{} problems", problems.len());
    }
    println!("{} wallpapers across {} themes check out.", inputs.rows.len(), manifest::by_theme(&inputs.rows).len());
    Ok(())
}
