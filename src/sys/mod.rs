use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, Read},
};

pub mod disks;
pub mod uptime;
pub mod format_num;
pub mod hostname;
pub mod os_release;
pub mod process;

pub use self::format_num::MemUnit;
pub use self::hostname::hostname;
pub use self::os_release::OsRelease;
pub use self::uptime::uptime;
pub use self::process::process_by_user;
pub use self::disks::get_all_disks;

// https://github.com/FillZpp/sys-info-rs
/// System memory information.
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
            let label = label.unwrap().split(':').nth(0).ok_or(Error::Unknown)?;
            let value = value.unwrap().parse::<u64>().ok().ok_or(Error::Unknown)?;
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
    match find_cpu_mhz {
        None => find_cpu_mhz = s.split('\n').find(|line| line.starts_with("BogoMIPS")),
        _ => {}
    }
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
                .and_then(|line| line.split('@').nth(0))
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
    let s = str::replace(&s, "Core2", "Core 2");
    s
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
