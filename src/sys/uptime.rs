extern crate libc;
use std::{fmt::Write, mem, time::Duration};

pub fn uptime() -> String {
    match get_uptime() {
        Ok(uptime) => format_duration(uptime),
        Err(err) => {
            eprintln!("Uptime: {}", err);
            std::process::exit(1);
        }
    }
}

pub fn get_uptime() -> Result<Duration, String> {
    let mut info: libc::sysinfo = unsafe { mem::zeroed() };
    let ret = unsafe { libc::sysinfo(&mut info) };
    if ret == 0 {
        Ok(Duration::from_secs(info.uptime as u64))
    } else {
        Err("sysinfo failed".to_string())
    }
}

pub fn format_duration(duration: Duration) -> String {
    let sec = duration.as_secs();

    let years = sec / 31_557_600; // 365.25d
    let sec = sec % 31_557_600;

    let months = sec / 2_630_016;
    let sec = sec % 2_630_016;

    let days = sec / 86_400;
    let sec = sec % 86_400;

    let hours = sec / 3_600;
    let sec = sec % 3_600;

    let minutes = sec / 60;
    let seconds = sec % 60;

    let mut s = String::new();

    if years > 0 {
        s.write_fmt(format_args!("{} year", years)).unwrap();

        if years > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if months > 0 {
        s.write_fmt(format_args!("{} month", months)).unwrap();

        if months > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if days > 0 {
        s.write_fmt(format_args!("{} day", days)).unwrap();

        if days > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if hours > 0 || (days > 0) && (minutes > 0 || seconds > 0) {
        s.write_fmt(format_args!("{} hour", hours)).unwrap();

        if hours > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if minutes > 0 || (hours > 0 && seconds > 0) {
        s.write_fmt(format_args!("{} minute", minutes)).unwrap();

        if minutes > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if seconds > 0 {
        s.write_fmt(format_args!("{} second", seconds)).unwrap();

        if seconds > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    debug_assert!(s.len() >= 2);

    if let Some(index) = s.as_str()[..(s.len() - 2)].rfind(", ") {
        s.insert_str(index + 2, "and ");
    }

    let len = s.len();

    let mut v = s.into_bytes();

    unsafe {
        v.set_len(len - 2);

        String::from_utf8_unchecked(v)
    }
}

/*
fn parse_for_shorthand_time(uptime: String) -> String {
    let newtime = str::replace(&uptime, "years", "y");
    let newtime = str::replace(&newtime, "year", "y");
    let newtime = str::replace(&newtime, "days", "d");
    let newtime = str::replace(&newtime, "day", "d");
    let newtime = str::replace(&newtime, "hours", "h");
    let newtime = str::replace(&newtime, "hour", "h");
    let newtime = str::replace(&newtime, "minutes", "m");
    let newtime = str::replace(&newtime, "minute", "m");
    let newtime = str::replace(&newtime, "seconds", "s");
    str::replace(&newtime, "second", "s")
}
*/
