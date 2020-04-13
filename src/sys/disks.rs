use libc::statvfs;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read, Seek},
    mem,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

// This source code was adopted from https://github.com/GuillaumeGomez/sysinfo

fn find_type_for_name(name: &OsStr) -> DiskType {
    /* turn "sda1" into "sda": */
    let mut trimmed: &[u8] = name.as_bytes();
    while trimmed.len() > 1
        && trimmed[trimmed.len() - 1] >= b'0'
        && trimmed[trimmed.len() - 1] <= b'9'
    {
        trimmed = &trimmed[..trimmed.len() - 1]
    }
    let trimmed: &OsStr = OsStrExt::from_bytes(trimmed);

    let path = Path::new("/sys/block/")
        .to_owned()
        .join(trimmed)
        .join("queue/rotational");
    // Normally, this file only contains '0' or '1' but just in case, we get 8 bytes...
    let rotational_int = get_all_data(path, 8).unwrap_or_default().trim().parse();
    DiskType::from(rotational_int.unwrap_or(-1))
}

macro_rules! cast {
    ($x:expr) => {
        u64::from($x)
    };
}

pub fn new_disk(name: &OsStr, mount_point: &Path, file_system: &[u8]) -> Disk {
    let mount_point_cpath = to_cpath(mount_point);
    let type_ = find_type_for_name(name);
    let mut total = 0;
    let mut available = 0;
    unsafe {
        let mut stat: statvfs = mem::zeroed();
        if statvfs(mount_point_cpath.as_ptr() as *const _, &mut stat) == 0 {
            total = cast!(stat.f_bsize) * cast!(stat.f_blocks);
            available = cast!(stat.f_bsize) * cast!(stat.f_bavail);
        }
    }
    Disk {
        type_,
        name: name.to_owned(),
        file_system: file_system.to_owned(),
        mount_point: mount_point.to_owned(),
        total_space: cast!(total),
        available_space: cast!(available),
    }
}

/// Struct containing a disk information.
pub struct Disk {
    pub type_: DiskType,
    pub name: OsString,
    pub file_system: Vec<u8>,
    pub mount_point: PathBuf,
    pub total_space: u64,
    pub available_space: u64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiskType {
    /// HDD type.
    HDD,
    /// SSD type.
    SSD,
    /// Unknown type.
    Unknown(isize),
}

impl From<isize> for DiskType {
    fn from(t: isize) -> DiskType {
        match t {
            0 => DiskType::HDD,
            1 => DiskType::SSD,
            id => DiskType::Unknown(id),
        }
    }
}
pub trait DiskExt {
    /// Returns the disk type.
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{:?}", disk.get_type());
    /// }
    /// ```
    fn get_type(&self) -> DiskType;

    /// Returns the disk name.
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{:?}", disk.get_name());
    /// }
    /// ```
    fn get_name(&self) -> &OsStr;

    /// Returns the file system used on this disk (so for example: `EXT4`, `NTFS`, etc...).
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{:?}", disk.get_file_system());
    /// }
    /// ```
    fn get_file_system(&self) -> &[u8];

    /// Returns the mount point of the disk (`/` for example).
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{:?}", disk.get_mount_point());
    /// }
    /// ```
    fn get_mount_point(&self) -> &Path;

    /// Returns the total disk size, in bytes.
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{}", disk.get_total_space());
    /// }
    /// ```
    fn get_total_space(&self) -> u64;

    /// Returns the available disk size, in bytes.
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let s = System::new();
    /// for disk in s.get_disks() {
    ///     println!("{}", disk.get_available_space());
    /// }
    /// ```
    fn get_available_space(&self) -> u64;

    /// Updates the disk' information.
    ///
    /// ```no_run
    /// use sysinfo::{DiskExt, System, SystemExt};
    ///
    /// let mut s = System::new_all();
    /// for disk in s.get_disks_mut() {
    ///     disk.refresh();
    /// }
    /// ```
    fn refresh(&mut self) -> bool;
}
impl DiskExt for Disk {
    fn get_type(&self) -> DiskType {
        self.type_
    }

    fn get_name(&self) -> &OsStr {
        &self.name
    }

    fn get_file_system(&self) -> &[u8] {
        &self.file_system
    }

    fn get_mount_point(&self) -> &Path {
        &self.mount_point
    }

    fn get_total_space(&self) -> u64 {
        self.total_space
    }

    fn get_available_space(&self) -> u64 {
        self.available_space
    }

    fn refresh(&mut self) -> bool {
        unsafe {
            let mut stat: statvfs = mem::zeroed();
            let mount_point_cpath = to_cpath(&self.mount_point);
            if statvfs(mount_point_cpath.as_ptr() as *const _, &mut stat) == 0 {
                let tmp = cast!(stat.f_bsize) * cast!(stat.f_bavail);
                self.available_space = cast!(tmp);
                true
            } else {
                false
            }
        }
    }
}

pub fn get_all_disks() -> Vec<Disk> {
    let content = get_all_data("/proc/mounts", 16_385).unwrap_or_default();
    let disks = content.lines().filter(|line| {
        let line = line.trim_start();
        // While the `sd` prefix is most common, some disks instead use the `nvme` prefix. This
        // prefix refers to NVM (non-volatile memory) cabale SSDs. These disks run on the NVMe
        // storage controller protocol (not the scsi protocol) and as a result use a different
        // prefix to support NVMe namespaces.
        //
        // In some other cases, it uses a device mapper to map physical block devices onto
        // higher-level virtual block devices (on `/dev/mapper`).
        //
        // Raspbian uses root and mmcblk for physical disks
        line.starts_with("/dev/sd")
            || line.starts_with("/dev/nvme")
            || line.starts_with("/dev/mapper/")
            || line.starts_with("/dev/root")
            || line.starts_with("/dev/mmcblk")
    });
    let mut ret = vec![];

    for line in disks {
        let mut split = line.split(' ');
        if let (Some(name), Some(mountpt), Some(fs)) = (split.next(), split.next(), split.next()) {
            ret.push(new_disk(
                name[5..].as_ref(),
                Path::new(mountpt),
                fs.as_bytes(),
            ));
        }
    }
    ret
}

pub fn to_cpath(path: &Path) -> Vec<u8> {
    let path_os: &OsStr = path.as_ref();
    let mut cpath = path_os.as_bytes().to_vec();
    cpath.push(0);
    cpath
}

fn get_all_data_from_file(file: &mut File, size: usize) -> io::Result<String> {
    let mut data = Vec::with_capacity(size);
    unsafe {
        data.set_len(size);
    }

    file.seek(::std::io::SeekFrom::Start(0))?;
    let size = file.read(&mut data)?;
    data.truncate(size);
    Ok(String::from_utf8(data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?)
}
pub fn get_all_data<P: AsRef<Path>>(file_path: P, size: usize) -> io::Result<String> {
    let mut file = File::open(file_path.as_ref())?;
    get_all_data_from_file(&mut file, size)
}
