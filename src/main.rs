pub mod quotes;
pub mod services;
pub mod sys;
#[cfg(feature = "tui")]
pub mod ui;
#[cfg(feature = "tui")]
pub mod util;

#[cfg(feature = "tui")]
use crate::{
    ui::{layout, App},
    util::event::{Config, Event, Events},
};

#[cfg(feature = "tui")]
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
#[cfg(feature = "tui")]
use tui::{backend::TermionBackend, Terminal};
#[cfg(feature = "tui")]
use std::{error::Error, io, time::Duration};

#[cfg(feature = "tui")]
fn main() -> Result<(), Box<dyn Error>> {
    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(10000),
        ..Config::default()
    });
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = App::new("Termion demo");
    loop {
        terminal.draw(|mut f| layout::draw(&mut f, &mut app))?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

#[cfg(feature = "terminal")]
use termion::{color, style, terminal_size};
#[cfg(feature = "terminal")]
use textwrap::fill;
#[cfg(feature = "terminal")]
use quotes::get_quote;
#[cfg(feature = "terminal")]
use services::{get_docker_processes, list_unit_files, systemd};
#[cfg(feature = "terminal")]
use sys::{
    cpu_info, get_all_disks, get_kernel, hostname, loadavg, mem_info, process_by_user, uptime,
    MemUnit, OsRelease,
};


#[cfg(feature = "terminal")]
fn main() {
    let (w, _h) = terminal_size().unwrap();
    let mem = mem_info().unwrap();
    let cpu = cpu_info().unwrap();
    let disks = get_all_disks();
    let load = loadavg().unwrap();
    let pbu = process_by_user();
    let quote = get_quote();

    println!(
        " - {}{}Hostname{}{}..: {}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        hostname().into_string().unwrap()
    );
    println!(
        " - {}{}Distro{}{}....: {}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        OsRelease::new().expect("Error Will Robinson").pretty_name
    );
    println!(
        " - {}{}Kernel{}{}....: {}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        get_kernel().unwrap()
    );
    println!();
    println!(
        " - {}{}Uptime{}{}....: {}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        uptime()
    );
    println!(
        " - {}{}Load{}{}......: {}{}{}{}{} (1m), {}{}{}{}{} (5m), {}{}{}{}{} (15m)",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        load.one.to_string(),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        load.five.to_string(),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        load.fifteen.to_string(),
        style::Reset,
        color::Fg(color::Reset),
    );
    println!(
        " - {}{}Proc{}{}......: {}{}{}{}{} (all), {}{}{}{}{} (root), {}{}{}{}{} (user)",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        pbu.all.to_string(),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        pbu.root.to_string(),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        pbu.user.to_string(),
        style::Reset,
        color::Fg(color::Reset),
    );
    println!();
    println!(
        " - {}{}CPU{}{}.......: {}({}/{}) @ {}MHz",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        cpu.name,
        cpu.physical_cores,
        cpu.logical_cores,
        cpu.speed
    );
    println!(
        " - {}{}Memory{}{}....: {}{}{}{}{} used, {}{}{}{}{} free, {}{}{}{}{} total",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        MemUnit::MiB(
            (mem.total - mem.free - mem.cached - mem.buffers - mem.sreclaimable) as f64 * 1024_f64
        ),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        MemUnit::MiB(mem.free as f64 * 1024_f64),
        style::Reset,
        color::Fg(color::Reset),
        color::Fg(color::Green),
        style::Bold,
        MemUnit::MiB(mem.total as f64 * 1024_f64),
        style::Reset,
        color::Fg(color::Reset)
    );

    println!(
        " - {}{}Volumes{}{}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset)
    );
    for disk in disks {
        if disk.total_space >= 1_000_000_000 {
            println!(
                "     {}{}{}",
                disk.mount_point.to_str().unwrap(),
                " ".repeat(
                    w as usize
                        - (disk.mount_point.to_str().unwrap().len()
                            + 13
                            + ((((disk.total_space - disk.available_space) as f64
                                / disk.total_space as f64)
                                * 100.0)
                                .ceil()
                                .to_string()
                                + "% out of "
                                + &format!("{}", MemUnit::GB(disk.total_space as f64)))
                                .len())
                ),
                (((disk.total_space - disk.available_space) as f64 / disk.total_space as f64)
                    * 100.0)
                    .ceil()
                    .to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::GB(disk.total_space as f64))
            );
        } else {
            println!(
                "     {}{}{}",
                disk.mount_point.to_str().unwrap(),
                " ".repeat(
                    w as usize
                        - (disk.mount_point.to_str().unwrap().len()
                            + 13
                            + ((((disk.total_space - disk.available_space) as f64
                                / disk.total_space as f64)
                                * 100.0)
                                .ceil()
                                .to_string()
                                + "% out of "
                                + &format!("{}", MemUnit::MB(disk.total_space as f64)))
                                .len())
                ),
                (((disk.total_space - disk.available_space) as f64 / disk.total_space as f64)
                    * 100.0)
                    .ceil()
                    .to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::MB(disk.total_space as f64))
            );
        }

        println!(
            "     [{}{}{}{}{}]",
            color::Fg(color::Blue),
            "=".repeat(
                ((w as f64 - 15.0)
                    * ((disk.total_space - disk.available_space) as f64 / disk.total_space as f64))
                    .ceil() as usize
            ),
            color::Fg(color::Magenta),
            "=".repeat(
                ((w as f64 - 15.0)
                    * (1.0
                        - ((disk.total_space - disk.available_space) as f64
                            / disk.total_space as f64)))
                    .floor() as usize
            ),
            color::Fg(color::Reset)
        )
    }
    println!(
        "\n - {}{}Systemd Services{}{}",
        color::Fg(color::Cyan),
        style::Bold,
        style::Reset,
        color::Fg(color::Reset)
    );
    for sd_unit in list_unit_files().unwrap() {
        if !sd_unit.name.is_empty()
            && vec![
                "fail2ban",
                "plexmediaserver",
                "samba",
                "smartd",
                "smbd",
                "sshd",
                "ufw",
            ]
            .contains(&sd_unit.name.as_str())
        {
            match sd_unit.state {
                systemd::UnitState::Enabled | systemd::UnitState::EnabledRuntime => println!(
                    "     {}{} {}{}{}",
                    color::Fg(color::Green),
                    style::Bold,
                    sd_unit.name,
                    style::Bold,
                    style::Reset
                ),
                systemd::UnitState::Masked
                | systemd::UnitState::MaskedRuntime
                | systemd::UnitState::Disabled
                | systemd::UnitState::Bad => println!(
                    "     {}{} {}{}{}",
                    color::Fg(color::Red),
                    style::Bold,
                    sd_unit.name,
                    style::Bold,
                    style::Reset
                ),
                _ => println!(
                    "     {}{}卑 {}{}{}",
                    color::Fg(color::Yellow),
                    style::Bold,
                    sd_unit.name,
                    style::Bold,
                    style::Reset
                ),
            }
        }
    }

    if let Some(processes) = get_docker_processes() {
        println!(
            "\n - {}{}Docker Containers{}{}",
            color::Fg(color::Cyan),
            style::Bold,
            style::Reset,
            color::Fg(color::Reset)
        );
        for process in processes {
            println!("     {} {}", process.0, process.1);
        }
    }

    println!("\n{}", fill(quote.quote, w as usize));
    // Print Author
    println!("\n\t- {}", quote.author);
}
