use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, Read},
};

pub mod disks;
pub mod format_num;
pub mod hostname;
pub mod os_release;
pub mod process;
pub mod uptime;

pub use self::disks::get_all_disks;
pub use self::format_num::MemUnit;
pub use self::hostname::hostname;
pub use self::os_release::{distro, OsRelease};
pub use self::process::{process_by_user, username};
pub use self::uptime::{format_duration, get_uptime, uptime};

// https://github.com/FillZpp/sys-info-rs
// System memory information.
#[derive(Debug)]
pub struct MemInfo {
    /// Total physical memory.
    pub total: u64,
    pub free: u64,
    pub cached: u64,
    pub buffers: u64,
    pub sreclaimable: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CpuInfo {
    pub speed: u64,
    pub physical_cores: u64,
    pub logical_cores: u64,
    pub name: String,
}

#[repr(C)]
#[derive(Debug)]
pub struct LoadAvg {
    /// Average load within one minite.
    pub one: f64,
    /// Average load within five minites.
    pub five: f64,
    /// Average load within fifteen minites.
    pub fifteen: f64,
}

pub fn mem_info() -> Result<MemInfo, Error> {
    let mut s = String::new();
    File::open("/proc/meminfo")?.read_to_string(&mut s)?;
    let mut meminfo_hashmap = HashMap::new();
    for line in s.lines() {
        let mut split_line = line.split_whitespace();
        let label = split_line.next();
        let value = split_line.next();
        if value.is_some() && label.is_some() {
            let label = match label {
                Some(a) => a,
                _ => unreachable!(),
            }
            .split(':')
            .next()
            .ok_or(Error::Unknown)?;
            let value = match value {
                Some(a) => a,
                _ => unreachable!(),
            }
            .parse::<u64>()
            .ok()
            .ok_or(Error::Unknown)?;
            meminfo_hashmap.insert(label, value);
        }
    }
    Ok(MemInfo {
        total: *meminfo_hashmap.get("MemTotal").ok_or(Error::Unknown)?,
        free: *meminfo_hashmap.get("MemFree").ok_or(Error::Unknown)?,
        cached: *meminfo_hashmap.get("Cached").ok_or(Error::Unknown)?,
        buffers: *meminfo_hashmap.get("Buffers").ok_or(Error::Unknown)?,
        sreclaimable: *meminfo_hashmap.get("SReclaimable").ok_or(Error::Unknown)?,
        swap_total: *meminfo_hashmap.get("SwapTotal").ok_or(Error::Unknown)?,
        swap_free: *meminfo_hashmap.get("SwapFree").ok_or(Error::Unknown)?,
    })
}

pub fn get_kernel() -> Result<String, Error> {
    let mut s = String::new();
    File::open("/proc/sys/kernel/osrelease")?.read_to_string(&mut s)?;
    s.pop(); // pop '\n'
    Ok(s)
}

pub fn cpu_info() -> Result<CpuInfo, Error> {
    let mut s = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut s)?;

    let find_cpu_model_name = s.split('\n').find(|line| line.starts_with("model name"));
    let find_logical_cores = s.split('\n').find(|line| line.starts_with("siblings"));
    let find_physical_cores = s.split('\n').find(|line| line.starts_with("cpu cores"));
    let mut find_cpu_mhz = s.split('\n').find(|line| line.starts_with("cpu MHz"));
    if find_cpu_mhz.is_none() {
        find_cpu_mhz = s.split('\n').find(|line| line.starts_with("BogoMIPS"))
    };
    Ok(CpuInfo {
        speed: find_cpu_mhz
            .and_then(|line| line.split(':').last())
            .and_then(|val| val.trim().parse::<f64>().ok())
            .map(|speed| speed as u64)
            .unwrap(),
        physical_cores: find_physical_cores
            .and_then(|line| line.split(':').last())
            .and_then(|val| val.trim().parse::<f64>().ok())
            .map(|cores| cores as u64)
            .unwrap(),
        logical_cores: find_logical_cores
            .and_then(|line| line.split(':').last())
            .and_then(|val| val.trim().parse::<f64>().ok())
            .map(|cores| cores as u64)
            .unwrap(),
        name: cpu_parse(
            find_cpu_model_name
                .and_then(|line| line.split(':').last())
                .and_then(|line| line.split('@').next())
                .unwrap()
                .trim(),
        ),
    })
}

fn cpu_parse(s: &str) -> String {
    let s = str::replace(&s, "(TM)", "");
    let s = str::replace(&s, "(tm)", "");
    let s = str::replace(&s, "(R)", "");
    let s = str::replace(&s, "(r)", "");
    let s = str::replace(&s, "CPU", "");
    let s = str::replace(&s, "Processor", "");
    let s = str::replace(&s, "Dual-Core", "");
    let s = str::replace(&s, "Quad-Core", "");
    let s = str::replace(&s, "Six-Core", "");
    let s = str::replace(&s, "Eight-Core", "");
    let s = str::replace(&s, "Compute Cores", "");
    //let s = str::replace(&s, "Core ", "");
    let s = str::replace(&s, "AuthenticAMD", "");
    let s = str::replace(&s, "with Radeon", "");
    let s = str::replace(&s, "Graphics", "");
    let s = str::replace(&s, "altivec supported", "");
    let s = str::replace(&s, "FPU", "");
    let s = str::replace(&s, "Chip Revision", "");
    let s = str::replace(&s, "Technologies, Inc", "");
    str::replace(&s, "Core2", "Core 2")
}

pub fn loadavg() -> Result<LoadAvg, Error> {
    let mut s = String::new();
    File::open("/proc/loadavg")?.read_to_string(&mut s)?;
    let loads = s
        .trim()
        .split(' ')
        .take(3)
        .map(|val| val.parse::<f64>().unwrap())
        .collect::<Vec<f64>>();
    Ok(LoadAvg {
        one: loads[0],
        five: loads[1],
        fifteen: loads[2],
    })
}
/// Error types
#[derive(Debug)]
pub enum Error {
    UnsupportedSystem,
    ExecFailed(io::Error),
    IO(io::Error),
    SystemTime(std::time::SystemTimeError),
    General(Box<dyn std::error::Error>),
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            UnsupportedSystem => write!(fmt, "System is not supported"),
            ExecFailed(ref e) => write!(fmt, "Execution failed: {}", e),
            IO(ref e) => write!(fmt, "IO error: {}", e),
            SystemTime(ref e) => write!(fmt, "System time error: {}", e),
            General(ref e) => write!(fmt, "Error: {}", e),
            Unknown => write!(fmt, "An unknown error occurred"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            UnsupportedSystem => "unsupported system",
            ExecFailed(_) => "execution failed",
            IO(_) => "io error",
            SystemTime(_) => "system time",
            General(_) => "general error",
            Unknown => "unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use self::Error::*;
        match *self {
            UnsupportedSystem => None,
            ExecFailed(ref e) => Some(e),
            IO(ref e) => Some(e),
            SystemTime(ref e) => Some(e),
            General(ref e) => Some(e.as_ref()),
            Unknown => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::SystemTime(e)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Error {
        Error::General(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{collections::BTreeMap, env, iter::FromIterator, process::Command, time::Duration};

    const EXAMPLE_OSRELEASE: &str = r#"NAME="Pop!_OS"
    VERSION="18.04 LTS"
    ID=ubuntu
    ID_LIKE=debian
    PRETTY_NAME="Pop!_OS 18.04 LTS"
    VERSION_ID="18.04"
    HOME_URL="https://system76.com/pop"
    SUPPORT_URL="http://support.system76.com"
    BUG_REPORT_URL="https://github.com/pop-os/pop/issues"
    PRIVACY_POLICY_URL="https://system76.com/privacy"
    VERSION_CODENAME=bionic
    EXTRA_KEY=thing
    ANOTHER_KEY="#;
    #[test]
    pub fn test_loadavg() {
        let load = loadavg().unwrap();
        println!("loadavg(): {:?}", load);
    }
    #[test]
    pub fn test_mem_info() {
        let mem = mem_info().unwrap();
        assert!(mem.total > 0);
        println!("Mem: {:?}", mem);
    }
    #[test]
    pub fn test_cpu_info() {
        let cpu = cpu_info().unwrap();
        assert!(cpu.speed > 0);
        assert!(cpu.physical_cores > 0);
        assert!(cpu.logical_cores > 0);
        println!("Cpu: {:?}", cpu);
    }

    #[test]
    pub fn test_get_kernel() {
        let kernel = get_kernel().unwrap();
        assert!(!kernel.is_empty());
        println!("Kernel: {:?}", kernel);
    }

    #[test]
    fn gethostname_matches_system_hostname() {
        match Command::new("hostname").output() {
            Ok(x) => {
                let hostname = String::from_utf8_lossy(&x.stdout);
                // Convert both sides to lowercase; hostnames are case-insensitive
                // anyway.
                assert_eq!(
                    super::hostname().into_string().unwrap().to_lowercase(),
                    hostname.trim_end().to_lowercase()
                );
            }
            Err(_) => match Command::new("hostnamectl").output() {
                Ok(x) => {
                    let hostname = String::from_utf8_lossy(&x.stdout);
                    let hostname = hostname
                        .split('\n')
                        .find(|line| line.trim().starts_with("Static hostname"))
                        .and_then(|line| line.split(':').last())
                        .unwrap()
                        .trim();
                    assert_eq!(
                        super::hostname().into_string().unwrap().to_lowercase(),
                        hostname.trim_end().to_lowercase()
                    );
                }
                Err(e) => eprint!("Error: {}", e),
            },
        }
    }

    #[test]
    fn test_os_release() {
        let os_release = OsRelease::from_iter(EXAMPLE_OSRELEASE.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "Pop!_OS".into(),
                version: "18.04 LTS".into(),
                id: "ubuntu".into(),
                id_like: "debian".into(),
                pretty_name: "Pop!_OS 18.04 LTS".into(),
                version_id: "18.04".into(),
                home_url: "https://system76.com/pop".into(),
                support_url: "http://support.system76.com".into(),
                bug_report_url: "https://github.com/pop-os/pop/issues".into(),
                privacy_policy_url: "https://system76.com/privacy".into(),
                version_codename: "bionic".into(),
                extra: {
                    let mut map = BTreeMap::new();
                    map.insert("EXTRA_KEY".to_owned(), "thing".to_owned());
                    map
                }
            }
        )
    }

    #[test]
    fn test_distro() {
        let distro = distro();
        assert_ne!(distro, "Unknown".to_string());
    }
    #[test]
    fn test_uptime_get() {
        assert_eq!(get_uptime().is_ok(), true);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(
            format_duration(Duration::new(864_000_000, 0)).len() == 65,
            true
        );
        assert_eq!(
            format_duration(Duration::new(864_000_000, 0))
                == "27 years, 4 months, 16 days, 11 hours, 45 minutes, and 36 seconds",
            true
        );
        println!("{}", format_duration(Duration::new(864_000_000, 0)));
        println!(
            "len = {}",
            format_duration(Duration::new(864_000_000, 0)).len()
        );
    }

    #[test]
    fn test_uptime() {
        let d = get_uptime();
        println!("{:#?}", d);
        println!("len = {}", format_duration(d.unwrap()));
    }

    #[test]
    fn test_bytes() {
        assert_eq!(MemUnit::B(100.0).to_string(), String::from("100B"));
    }
    #[test]
    fn test_kilobytes() {
        assert_eq!(
            MemUnit::KB(1_000_000.0).to_string(),
            String::from("1,000KB")
        );
    }
    #[test]
    fn test_kibibytes() {
        assert_eq!(
            MemUnit::KiB(1_048_576.0).to_string(),
            String::from("1,024KiB")
        );
    }
    #[test]
    fn test_megabytes() {
        assert_eq!(
            MemUnit::MB(10_000_000_000.0).to_string(),
            String::from("10,000MB")
        );
    }
    #[test]
    fn test_mebibytes() {
        assert_eq!(
            MemUnit::MiB(12_073_741_824.0).to_string(),
            String::from("11,514MiB")
        );
    }

    #[test]
    fn test_gigabytes() {
        assert_eq!(
            MemUnit::GB(100_000_000_000_000.0).to_string(),
            String::from("100,000GB")
        );
    }
    #[test]
    fn test_gibibytes() {
        assert_eq!(
            MemUnit::GiB(244_099_511_627_776.0).to_string(),
            String::from("227,335GiB")
        );
    }
    #[test]
    fn test_terabytes() {
        assert_eq!(
            MemUnit::TB(100_000_000_000_000_000.0).to_string(),
            String::from("100,000TB")
        );
    }
    #[test]
    fn test_tebibytes() {
        assert_eq!(
            MemUnit::TiB(256_384_512_768_896_128.0).to_string(),
            String::from("233,180TiB")
        );
    }
    #[test]
    fn test_petabytes() {
        assert_eq!(
            MemUnit::PB(100_000_000_000_000_000_000.0).to_string(),
            String::from("100,000PB")
        );
    }
    #[test]
    fn test_pebibytes() {
        assert_eq!(
            MemUnit::PiB(128_256_384_512_768_896_128.0).to_string(),
            String::from("113,914PiB")
        );
    }

    #[test]
    fn test_processes() {
        let process_by_user = process_by_user();
        let mut root: usize = 0;
        let mut user: usize = 0;
        let mut count: usize = 0;
        match Command::new("ps").arg("-eo").arg("user").output() {
            Ok(x) => {
                for line in String::from_utf8(x.stdout).unwrap().lines().skip(1) {
                    if line.trim() == "root" {
                        root += 1
                    } else if line.trim() == env::var("USER").unwrap() {
                        user += 1
                    }
                    count += 1
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }

        // We subtract one from all and user because
        // we are creating one process calling `ps -eo user`
        assert_eq!(process_by_user.all, count - 1);
        assert_eq!(process_by_user.root, root);
        assert_eq!(process_by_user.user, user - 1);
    }
    #[test]
    fn test_username() {
        assert_eq!(username(), env::var("USER").unwrap())
    }

    #[test]
    fn test_get_all_disks() {
        let disk = get_all_disks();
        assert!(!disk.is_empty())
    }
}
