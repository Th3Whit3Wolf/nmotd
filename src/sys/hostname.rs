use std::ffi::OsString;
use std::io::Error;

// https://github.com/lunaryorn/gethostname.rs
#[inline]
pub fn hostname() -> OsString {
    use libc::{c_char, sysconf, _SC_HOST_NAME_MAX};
    use std::os::unix::ffi::OsStringExt;
    // Get the maximum size of host names on this system, and account for the
    // trailing NUL byte.
    let hostname_max = unsafe { sysconf(_SC_HOST_NAME_MAX) };
    let mut buffer = vec![0 as u8; (hostname_max as usize) + 1];
    let returncode = unsafe { libc::gethostname(buffer.as_mut_ptr() as *mut c_char, buffer.len()) };
    if returncode != 0 {
        // There are no reasonable failures, so lets panic
        panic!(
            "gethostname failed: {}
    Please report an issue to <https://github.com/lunaryorn/gethostname.rs/issues>!",
            Error::last_os_error()
        );
    }
    // We explicitly search for the trailing NUL byte and cap at the buffer
    // length: If the buffer's too small (which shouldn't happen since we
    // explicitly use the max hostname size above but just in case) POSIX
    // doesn't specify whether there's a NUL byte at the end, so if we didn't
    // check we might read from memory that's not ours.
    let end = buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or_else(|| buffer.len());
    buffer.resize(end, 0);
    OsString::from_vec(buffer)
}
