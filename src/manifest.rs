//! `wallpapers.tsv`: one row per wallpaper per theme.
//!
//! Columns: theme, SHA-256, installed file name, source. The source is an
//! HTTPS URL or a path under `images/` in this repository.

use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub const REPO: &str = "justcarlson/omarchy-wallpapers";
pub const RAW_BASE: &str = "https://raw.githubusercontent.com/justcarlson/omarchy-wallpapers/main";
pub const SITE: &str = "https://justcarlson.github.io/omarchy-wallpapers";

pub const HEADER: &str = "# theme\tsha256\tfile\tsource\n\
# Generated from the curator's manifest. Edit upstream, then run `gallery sync`.\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub theme: String,
    pub sha256: String,
    pub file: String,
    pub source: String,
}

/// Painters whose public-domain works are in the collection, by file prefix.
const ARTISTS: &[(&str, &str)] = &[
    ("alexandre-cabanel-", "Alexandre Cabanel"),
    ("john-martin-", "John Martin"),
    ("sargent-", "John Singer Sargent"),
];

const SMALL_WORDS: &[&str] = &["a", "an", "and", "at", "by", "for", "in", "of", "on", "or", "the", "to", "with"];

pub struct Credit {
    pub label: String,
    pub url: String,
}

impl Row {
    /// The file name without `curated-` and the extension. Unique per theme.
    pub fn slug(&self) -> &str {
        let stem = self.file.rsplit_once('.').map_or(self.file.as_str(), |(s, _)| s);
        stem.strip_prefix("curated-").unwrap_or(stem)
    }

    /// The HTTPS URL to fetch. `images/` rows resolve against this repository.
    pub fn url(&self) -> String {
        if self.source.starts_with("https://") {
            self.source.clone()
        } else {
            format!("{RAW_BASE}/{}", self.source)
        }
    }

    pub fn local(&self) -> bool {
        !self.source.starts_with("https://")
    }

    pub fn artist(&self) -> Option<&'static str> {
        ARTISTS.iter().find(|(prefix, _)| self.slug().starts_with(prefix)).map(|(_, name)| *name)
    }

    /// The GitHub owner and repository for a raw.githubusercontent.com source.
    fn github(&self) -> Option<(&str, &str, &str)> {
        let rest = self.source.strip_prefix("https://raw.githubusercontent.com/")?;
        let mut parts = rest.splitn(4, '/');
        Some((parts.next()?, parts.next()?, rest))
    }

    pub fn title(&self) -> String {
        let mut words = self.slug();
        if let Some((prefix, _)) = ARTISTS.iter().find(|(p, _)| words.starts_with(p)) {
            words = &words[prefix.len()..];
        } else if let Some((owner, _, _)) = self.github() {
            let owner: String = owner.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
            words = words.strip_prefix(&format!("{owner}-")).unwrap_or(words);
        }
        title_case(words)
    }

    pub fn credit(&self) -> Credit {
        if let Some((owner, repo, rest)) = self.github() {
            let mut parts = rest.splitn(4, '/');
            let (_, _, commit, path) = (parts.next(), parts.next(), parts.next().unwrap_or("main"), parts.next().unwrap_or(""));
            return Credit {
                label: format!("{owner}/{repo}"),
                url: format!("https://github.com/{owner}/{repo}/blob/{commit}/{path}"),
            };
        }
        if self.source.starts_with("https://upload.wikimedia.org/") {
            let name = commons_file_name(&self.source);
            return Credit {
                label: self.artist().map_or("Wikimedia Commons".into(), |a| format!("{a}, Wikimedia Commons")),
                url: format!("https://commons.wikimedia.org/wiki/File:{name}"),
            };
        }
        if self.local() {
            return Credit {
                label: self.artist().map_or("This repository".into(), |a| format!("{a}, restored here")),
                url: format!("https://github.com/{REPO}/blob/main/{}", self.source),
            };
        }
        Credit { label: host(&self.source).to_string(), url: self.source.clone() }
    }
}

fn host(url: &str) -> &str {
    url.split('/').nth(2).unwrap_or(url)
}

/// `…/commons/thumb/0/04/Name.jpg/3840px-Name.jpg` and `…/commons/a/ad/Name.jpg` both give `Name.jpg`.
fn commons_file_name(url: &str) -> String {
    let after = url.split("/commons/").nth(1).unwrap_or(url);
    let parts: Vec<&str> = after.split('/').collect();
    let name = if parts.first() == Some(&"thumb") { parts.get(3) } else { parts.get(2) };
    name.copied().unwrap_or(after).to_string()
}

fn title_case(words: &str) -> String {
    let words: Vec<&str> = words.split(['-', '_']).filter(|w| !w.is_empty()).collect();
    let last = words.len().saturating_sub(1);
    words
        .iter()
        .enumerate()
        .map(|(i, w)| {
            if i != 0 && i != last && SMALL_WORDS.contains(w) {
                return (*w).to_string();
            }
            let mut chars = w.chars();
            chars.next().map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse(text: &str) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [theme, sha256, file, source] = fields[..] else {
            bail!("line {}: want 4 tab-separated fields, got {}", index + 1, fields.len());
        };
        rows.push(Row { theme: theme.into(), sha256: sha256.into(), file: file.into(), source: source.into() });
    }
    Ok(rows)
}

pub fn read(path: &Path) -> Result<Vec<Row>> {
    parse(&std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?)
}

pub fn write(path: &Path, rows: &[Row]) -> Result<()> {
    let mut text = HEADER.to_string();
    for r in rows {
        text.push_str(&format!("{}\t{}\t{}\t{}\n", r.theme, r.sha256, r.file, r.source));
    }
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

/// Rules every manifest must meet. The installer and the gallery rely on them.
pub fn validate(rows: &[Row]) -> Result<()> {
    ensure!(!rows.is_empty(), "manifest is empty");
    let mut seen = HashSet::new();
    let mut slugs = HashSet::new();
    for r in rows {
        let at = format!("{}/{}", r.theme, r.file);
        ensure!(valid_name(&r.theme), "{at}: theme must be lowercase letters, digits and dashes");
        ensure!(r.sha256.len() == 64 && r.sha256.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()), "{at}: bad SHA-256");
        ensure!(r.file.starts_with("curated-") && !r.file.contains('/'), "{at}: file must start with curated- and have no slash");
        let ext = r.file.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        ensure!(matches!(ext, "jpg" | "jpeg" | "png" | "webp"), "{at}: unsupported image type");
        ensure!(seen.insert(at.clone()), "{at}: duplicate row");
        ensure!(slugs.insert(format!("{}/{}", r.theme, r.slug())), "{at}: two files share a name");
        if r.local() {
            ensure!(r.source.starts_with("images/") && !r.source.contains(".."), "{at}: local source must be under images/");
        } else {
            ensure!(r.source.starts_with("https://") && !r.source.contains(char::is_whitespace), "{at}: source must be an HTTPS URL");
        }
    }
    Ok(())
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Rows grouped by theme, in manifest order inside each theme.
pub fn by_theme(rows: &[Row]) -> BTreeMap<&str, Vec<&Row>> {
    let mut map: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
    for r in rows {
        map.entry(r.theme.as_str()).or_default().push(r);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(file: &str, source: &str) -> Row {
        Row { theme: "gruvbox".into(), sha256: "a".repeat(64), file: file.into(), source: source.into() }
    }

    #[test]
    fn titles_drop_collection_and_artist_prefixes() {
        let r = row("curated-apeiros-46b-mist-forest-1.png", "https://raw.githubusercontent.com/Apeiros-46B/everforest-walls/8f53/nature/mist_forest_1.png");
        assert_eq!(r.title(), "Mist Forest 1");
        assert_eq!(r.credit().label, "Apeiros-46B/everforest-walls");
        assert_eq!(r.credit().url, "https://github.com/Apeiros-46B/everforest-walls/blob/8f53/nature/mist_forest_1.png");
        let r = row("curated-dharmx-a-lighthouse-in-front-of-a-building.jpg", "https://raw.githubusercontent.com/dharmx/walls/6b/x.jpg");
        assert_eq!(r.title(), "A Lighthouse in Front of a Building");
        let r = row("curated-sargent-at-chioggia.jpg", "https://upload.wikimedia.org/wikipedia/commons/a/ad/John_S._Sargent_-_At_Chioggia.jpg");
        assert_eq!(r.title(), "At Chioggia");
        assert_eq!(r.credit().url, "https://commons.wikimedia.org/wiki/File:John_S._Sargent_-_At_Chioggia.jpg");
        let r = row("curated-john-martin-dover.jpg", "https://upload.wikimedia.org/wikipedia/commons/thumb/6/6a/Dover.jpg/1920px-Dover.jpg");
        assert_eq!(r.credit().url, "https://commons.wikimedia.org/wiki/File:Dover.jpg");
        assert_eq!(r.credit().label, "John Martin, Wikimedia Commons");
    }

    #[test]
    fn local_images_resolve_to_this_repository() {
        let r = row("curated-alexandre-cabanel-ophelia.jpg", "images/alexandre-cabanel-ophelia.jpg");
        assert_eq!(r.title(), "Ophelia");
        assert_eq!(r.url(), format!("{RAW_BASE}/images/alexandre-cabanel-ophelia.jpg"));
        assert!(validate(&[r]).is_ok());
    }

    #[test]
    fn validation_rejects_unsafe_rows() {
        assert!(validate(&[row("curated-x.jpg", "http://example.com/x.jpg")]).is_err());
        assert!(validate(&[row("curated-x.jpg", "images/../x.jpg")]).is_err());
        assert!(validate(&[row("x.jpg", "https://example.com/x.jpg")]).is_err());
        assert!(validate(&[row("curated-x.svg", "https://example.com/x.svg")]).is_err());
        let ok = row("curated-x.jpg", "https://example.com/x.jpg");
        assert!(validate(&[ok.clone(), ok]).is_err());
    }

    #[test]
    fn parse_skips_comments_and_round_trips() {
        let text = format!("{HEADER}gruvbox\t{}\tcurated-x.jpg\thttps://e.com/x.jpg\n", "a".repeat(64));
        let rows = parse(&text).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(parse("a\tb\n").is_err());
    }
}
