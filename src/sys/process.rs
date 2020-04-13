use std::{ffi::c_void, mem};

#[repr(C)]
struct PassWd {
    pw_name: *const c_void,
    pw_passwd: *const c_void,
    pw_uid: u32,
    pw_gid: u32,
    pw_gecos: *const c_void,
    pw_dir: *const c_void,
    pw_shell: *const c_void,
}

pub struct ProcessByUser {
    pub root: usize,
    pub user: usize,
    pub all: usize,
}

pub fn process_by_user() -> ProcessByUser {
    let process_list = procfs::process::all_processes().expect("Error getting list of processes");

    let mut root: usize = 0;
    let mut user: usize = 0;
    for process in &process_list {
        if process.owner == 0 {
            root += 1
        } else if process.owner == unsafe { geteuid() } {
            user += 1
        }
    }

    ProcessByUser {
        root,
        user,
        all: process_list.len(),
    }
}
extern "system" {
    fn getpwuid_r(
        uid: u32,
        pwd: *mut PassWd,
        buf: *mut c_void,
        buflen: usize,
        result: *mut *mut PassWd,
    ) -> i32;
    fn geteuid() -> u32;
    fn strlen(cs: *const c_void) -> usize;
}

#[inline(always)]
fn getpwuid() -> (String, String) {
    const BUF_SIZE: usize = 16_384; // size from the man page
    let mut buffer = mem::MaybeUninit::<[u8; BUF_SIZE]>::uninit();
    let mut passwd = mem::MaybeUninit::<PassWd>::uninit();
    let mut _passwd = mem::MaybeUninit::<*mut PassWd>::uninit();

    // Get PassWd `struct`.
    let passwd = unsafe {
        getpwuid_r(
            geteuid(),
            passwd.as_mut_ptr(),
            buffer.as_mut_ptr() as *mut c_void,
            BUF_SIZE,
            _passwd.as_mut_ptr(),
        );

        passwd.assume_init()
    };

    // Extract names.
    let a = string_from_cstring(passwd.pw_name);
    let b = string_from_cstring(passwd.pw_gecos);

    (a, b)
}

fn string_from_cstring(string: *const c_void) -> String {
    if string.is_null() {
        return "".to_string();
    }

    // Get a byte slice of the c string.
    let slice = unsafe {
        let length = strlen(string);
        std::slice::from_raw_parts(string as *const u8, length)
    };

    // Turn byte slice into Rust String.
    String::from_utf8_lossy(slice).to_string()
}

pub fn username() -> String {
    let pwent = getpwuid();
    pwent.0
}
