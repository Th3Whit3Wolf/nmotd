use std::fs;
use std::io;

const KERNEL_PATH: &str = "/proc/sys/kernel/hostname";
#[inline]
pub fn get_kernel() -> Result<String, io::Error> {
    let mut s = fs::read_to_string(KERNEL_PATH)?;

    if s.ends_with('\n') {
        s.remove(s.len() - 1);
    }

    Ok(s)
}