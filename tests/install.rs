//! End-to-end tests for install.sh against a fake home directory.
//! The fixture serves images from a local folder, both as a checkout and
//! through `curl | bash` with a file:// base URL.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

struct Fixture {
    dir: PathBuf,
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl Fixture {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("omarchy-wallpapers-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("repo/images")).unwrap();
        fs::create_dir_all(dir.join("home/.config/omarchy/themes/gruvbox")).unwrap();
        fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh"), dir.join("repo/install.sh")).unwrap();
        let fixture = Fixture { dir };
        fixture.manifest(&[("gruvbox", "a"), ("gruvbox", "b"), ("nord", "c"), ("nord", "a")]);
        fixture
    }

    /// Rows of (theme, image); each image's bytes are its name.
    fn manifest(&self, rows: &[(&str, &str)]) {
        let mut text = String::from("# theme\tsha256\tfile\tsource\n");
        for (theme, image) in rows {
            fs::write(self.dir.join(format!("repo/images/{image}.jpg")), image).unwrap();
            text.push_str(&format!("{theme}\t{}\tcurated-{image}.jpg\timages/{image}.jpg\n", sha(image.as_bytes())));
        }
        fs::write(self.dir.join("repo/wallpapers.tsv"), text).unwrap();
    }

    fn home(&self) -> PathBuf {
        self.dir.join("home")
    }

    fn command(&self) -> Command {
        let mut command = Command::new("bash");
        command
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap())
            .env("HOME", self.home())
            .env("OMARCHY_PATH", self.dir.join("no-system-themes"));
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().arg(self.dir.join("repo/install.sh")).args(args).output().unwrap()
    }

    /// The way users run it: the script arrives on stdin, so it fetches the manifest.
    fn run_piped(&self, args: &[&str]) -> Output {
        let mut child = self
            .command()
            .env("OMARCHY_WALLPAPERS_URL", format!("file://{}", self.dir.join("repo").display()))
            .args(["-s", "--"])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let script = fs::read(self.dir.join("repo/install.sh")).unwrap();
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), &script).unwrap();
        drop(child.stdin.take());
        child.wait_with_output().unwrap()
    }

    fn installed(&self, theme: &str) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.home().join(".config/omarchy/backgrounds").join(theme))
            .map(|d| d.map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn ok(output: &Output) -> String {
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success(), "install.sh failed:\n{text}");
    text
}

#[test]
fn installs_only_the_themes_you_have_and_is_idempotent() {
    let f = Fixture::new("default");
    let text = ok(&f.run(&[]));
    assert!(text.contains("2 wallpapers ready across 1 themes"), "{text}");
    assert_eq!(f.installed("gruvbox"), ["curated-a.jpg", "curated-b.jpg"]);
    assert!(f.installed("nord").is_empty());
    ok(&f.run(&["--check"]));
    let again = ok(&f.run(&[]));
    assert!(again.contains("(0 new)"), "{again}");
}

#[test]
fn piped_install_fetches_the_manifest_and_images() {
    let f = Fixture::new("piped");
    let text = ok(&f.run_piped(&["--all"]));
    assert!(text.contains("4 wallpapers ready across 2 themes"), "{text}");
    assert_eq!(f.installed("nord"), ["curated-a.jpg", "curated-c.jpg"]);
    ok(&f.run_piped(&["--check", "--all"]));
}

#[test]
fn named_sets_install_without_the_theme() {
    let f = Fixture::new("named");
    ok(&f.run(&["nord"]));
    assert_eq!(f.installed("nord"), ["curated-a.jpg", "curated-c.jpg"]);
    assert!(f.installed("gruvbox").is_empty());
    let unknown = f.run(&["solarized"]);
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("no set named 'solarized'"));
}

#[test]
fn shared_images_are_stored_once() {
    let f = Fixture::new("links");
    ok(&f.run(&["--all"]));
    let backgrounds = f.home().join(".config/omarchy/backgrounds");
    use std::os::unix::fs::MetadataExt;
    let a = fs::metadata(backgrounds.join("gruvbox/curated-a.jpg")).unwrap();
    let b = fs::metadata(backgrounds.join("nord/curated-a.jpg")).unwrap();
    assert_eq!(a.ino(), b.ino());
}

#[test]
fn updates_remove_dropped_picks_but_keep_your_files() {
    let f = Fixture::new("update");
    ok(&f.run(&["--all"]));
    let mine = f.home().join(".config/omarchy/backgrounds/gruvbox/my-photo.jpg");
    fs::write(&mine, "mine").unwrap();
    f.manifest(&[("gruvbox", "a"), ("nord", "c")]);
    let check = f.run(&["--all", "--check"]);
    assert!(!check.status.success(), "check should report the dropped picks");
    ok(&f.run(&["--all"]));
    assert_eq!(f.installed("gruvbox"), ["curated-a.jpg", "my-photo.jpg"]);
    assert_eq!(f.installed("nord"), ["curated-c.jpg"]);
}

#[test]
fn installing_one_set_leaves_other_sets_alone() {
    let f = Fixture::new("partial");
    ok(&f.run(&["--all"]));
    ok(&f.run(&["gruvbox"]));
    assert_eq!(f.installed("nord"), ["curated-a.jpg", "curated-c.jpg"]);
    ok(&f.run(&["--all", "--check"]));
}

#[test]
fn a_bad_checksum_stops_the_file_from_installing() {
    let f = Fixture::new("checksum");
    fs::write(f.dir.join("repo/images/b.jpg"), "tampered").unwrap();
    let output = f.run(&["gruvbox"]);
    assert!(!output.status.success());
    assert_eq!(f.installed("gruvbox"), ["curated-a.jpg"]);
}

#[test]
fn remove_deletes_only_what_it_installed() {
    let f = Fixture::new("remove");
    ok(&f.run(&["--all"]));
    let mine = f.home().join(".config/omarchy/backgrounds/gruvbox/my-photo.jpg");
    fs::write(&mine, "mine").unwrap();
    let text = ok(&f.run(&["--remove"]));
    assert!(text.contains("Removed 4 wallpapers"), "{text}");
    assert_eq!(f.installed("gruvbox"), ["my-photo.jpg"]);
    assert!(f.installed("nord").is_empty());
    assert!(!f.home().join(".cache/omarchy-wallpapers").exists());
}

#[test]
fn list_shows_every_set() {
    let f = Fixture::new("list");
    let text = ok(&f.run(&["--list"]));
    assert!(text.contains("gruvbox") && text.contains("nord"), "{text}");
}
