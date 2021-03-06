// Type for parsing the `/etc/os-release` file.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader},
    iter::FromIterator,
    path::Path,
};
/*
lazy_static! {
    /// The OS release detected on this host's environment.
    ///
    /// # Notes
    /// If an OS Release was not found, an error will be in its place.
    pub static ref OS_RELEASE: io::Result<OsRelease> = OsRelease::new();
}
*/
macro_rules! map_keys {
    ($item:expr, { $($pat:expr => $field:expr),+ }) => {{
        $(
            if $item.starts_with($pat) {
                $field = parse_line($item, $pat.len()).into();
                continue;
            }
        )+
    }};
}

fn is_enclosed_with(line: &str, pattern: char) -> bool {
    line.starts_with(pattern) && line.ends_with(pattern)
}

fn parse_line(line: &str, skip: usize) -> &str {
    let line = line[skip..].trim();
    if is_enclosed_with(line, '"') || is_enclosed_with(line, '\'') {
        &line[1..line.len() - 1]
    } else {
        line
    }
}

/// Contents of the `/etc/os-release` file, as a data structure.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsRelease {
    /// The URL where bugs should be reported for this OS.
    pub bug_report_url: String,
    /// The homepage of this OS.
    pub home_url: String,
    /// Identifier of the original upstream OS that this release is a derivative of.
    ///
    /// **IE:** `debian`
    pub id_like: String,
    /// An identifier which describes this release, such as `ubuntu`.
    ///
    /// **IE:** `ubuntu`
    pub id: String,
    /// The name of this release, without the version string.
    ///
    /// **IE:** `Ubuntu`
    pub name: String,
    /// The name of this release, with th eversion stirng.
    ///
    /// **IE:** `Ubuntu 18.04 LTS`
    pub pretty_name: String,
    /// The URL describing this OS's privacy policy.
    pub privacy_policy_url: String,
    /// The URL for seeking support with this OS release.
    pub support_url: String,
    /// The codename of this version.
    ///
    /// **IE:** `bionic`
    pub version_codename: String,
    /// The version of this OS release, with additional details about the release.
    ///
    /// **IE:** `18.04 LTS (Bionic Beaver)`
    pub version_id: String,
    /// The version of this OS release.
    ///
    /// **IE:** `18.04`
    pub version: String,
    /// Additional keys not covered by the API.
    pub extra: BTreeMap<String, String>,
}

impl OsRelease {
    /// Attempt to parse the contents of `/etc/os-release`.
    pub fn new() -> io::Result<OsRelease> {
        let file = BufReader::new(open("/etc/os-release")?);
        Ok(OsRelease::from_iter(file.lines().flatten()))
    }

    /// Attempt to parse any `/etc/os-release`-like file.
    pub fn new_from<P: AsRef<Path>>(path: P) -> io::Result<OsRelease> {
        let file = BufReader::new(open(&path)?);
        Ok(OsRelease::from_iter(file.lines().flatten()))
    }
}

impl FromIterator<String> for OsRelease {
    fn from_iter<I: IntoIterator<Item = String>>(lines: I) -> Self {
        let mut os_release = Self::default();

        for line in lines {
            let line = line.trim();
            map_keys!(line, {
                "NAME=" => os_release.name,
                "VERSION=" => os_release.version,
                "ID=" => os_release.id,
                "ID_LIKE=" => os_release.id_like,
                "PRETTY_NAME=" => os_release.pretty_name,
                "VERSION_ID=" => os_release.version_id,
                "HOME_URL=" => os_release.home_url,
                "SUPPORT_URL=" => os_release.support_url,
                "BUG_REPORT_URL=" => os_release.bug_report_url,
                "PRIVACY_POLICY_URL=" => os_release.privacy_policy_url,
                "VERSION_CODENAME=" => os_release.version_codename
            });

            if let Some(pos) = line.find('=') {
                if line.len() > pos + 1 {
                    os_release
                        .extra
                        .insert(line[..pos].to_owned(), line[pos + 1..].to_owned());
                }
            }
        }

        os_release
    }
}

fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::open(&path).map_err(|why| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("unable to open file at {:?}: {}", path.as_ref(), why),
        )
    })
}

// Note Everything Above This is came from
// https://github.com/pop-os/os-release/blob/master/src/lib.rs
// Note new_from can parse any os-release type file
pub fn distro() -> String {
    if Path::new("/etc/os-release").is_file() {
        let release = OsRelease::new().expect("Error Will Robinson");
        release.pretty_name
    } else if Path::new("/etc/redstar-release").is_file() {
        String::from("Red Star OS")
    } else if Path::new("/etc/siduction-version").is_file() {
        String::from("Siductiond")
    } else if Path::new("/etc/GoboLinuxVersion").is_file() {
        String::from("GoboLinux")
    } else if Path::new("/etc/pcbsd-lang").is_file() {
        String::from("PCBSD")
    } else if Path::new("/etc/trueos-lang").is_file() {
        String::from("TrueOS")
    } else if Path::new("/bedrock/etc/bedrock-release").is_file() {
        String::from("Bedrock Linux")
    } else if Path::new("/etc/pacbsd-release").is_file() {
        String::from("PacBSD")
    } else {
        "Unknown".to_string()
    }
}
