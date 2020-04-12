use ansi_term::Colour::{Blue, Cyan, Green, Purple, Red, Yellow};
use terminal_size::{terminal_size, Height, Width};

pub mod disks;
pub mod docker;
pub mod format_num;
pub mod hostname;
pub mod os_release;
pub mod process;
pub mod sys;
pub mod systemd;
pub mod uptime;

use docker::get_docker_processes;
use format_num::MemType::*;
use hostname::hostname;
use os_release::OsRelease;
use sys::{cpu_info, get_kernel, loadavg, mem_info};
use systemd::list_unit_files;
use uptime::uptime;

use process::process_by_user;

use disks::get_all_disks;

fn main() {
    let size = terminal_size();
    let mem = mem_info().unwrap();
    let cpu = cpu_info().unwrap();
    let disks = get_all_disks();
    let load = loadavg().unwrap();
    let pbu = process_by_user();
    if let Some((Width(w), Height(_h))) = size {
        //println!("Your terminal is {} cols wide and lines tall", w);
        println!(
            " - {}..: {}",
            Cyan.bold().paint("Hostname"),
            hostname().into_string().unwrap()
        );
        println!(
            " - {}....: {}",
            Cyan.bold().paint("Distro"),
            OsRelease::new().expect("Error Will Robinson").pretty_name
        );
        println!(
            " - {}....: {}",
            Cyan.bold().paint("Kernel"),
            get_kernel().unwrap()
        );
        println!();
        println!(" - {}....: {}", Cyan.bold().paint("Uptime"), uptime());
        println!(
            " - {}......: {} (1m), {} (5m), {} (15m)",
            Cyan.bold().paint("Load"),
            Green.bold().paint(load.one.to_string()),
            Green.bold().paint(load.five.to_string()),
            Green.bold().paint(load.fifteen.to_string())
        );
        println!(
            " - {}......: {} (all), {} (root), {} (user)",
            Cyan.bold().paint("Proc"),
            Green.bold().paint(pbu.all.to_string()),
            Green.bold().paint(pbu.root.to_string()),
            Green.bold().paint(pbu.user.to_string())
        );
        println!();
        println!(
            " - {}.......: {}({}/{}) @ {}MHz",
            Cyan.bold().paint("CPU"),
            cpu.name,
            cpu.physical_cores,
            cpu.logical_cores,
            cpu.speed
        );
        println!(
            " - {}....: {} used, {} free, {} total",
            Cyan.bold().paint("Memory"),
            Green.bold().paint(
                MiB(
                    (mem.total - mem.free - mem.cached - mem.buffers - mem.sreclaimable) as f64
                        * 1024 as f64
                )
                .to_string()
            ),
            Green
                .bold()
                .paint(MiB(mem.free as f64 * 1024 as f64).to_string()),
            Green
                .bold()
                .paint(MiB(mem.total as f64 * 1024 as f64).to_string())
        );

        println!(" - {}", Cyan.bold().paint("Volumes"));
        for disk in disks {
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
                                + &format!("{}", GB(disk.total_space as f64)))
                                .len())
                ),
                (((disk.total_space - disk.available_space) as f64 / disk.total_space as f64)
                    * 100.0)
                    .ceil()
                    .to_string()
                    + "% out of "
                    + &format!("{}", GB(disk.total_space as f64))
            );
            println!(
                "        [{}{}]",
                Blue.bold().paint(
                    "=".repeat(
                        ((w as f64 - 18.0)
                            * ((disk.total_space - disk.available_space) as f64
                                / disk.total_space as f64))
                            .ceil() as usize
                    )
                ),
                Purple.paint(
                    "=".repeat(
                        ((w as f64 - 18.0)
                            * (1.0
                                - ((disk.total_space - disk.available_space) as f64
                                    / disk.total_space as f64)))
                            .floor() as usize
                    )
                )
            )
        }
        println!("\n - {}", Cyan.bold().paint("Systemd Services"));
        for sd_unit in list_unit_files(vec!["jellyfin", "sshd", "ufw"]).unwrap() {
            if !sd_unit.name.is_empty() {
                match sd_unit.state {
                    systemd::UnitState::Enabled | systemd::UnitState::EnabledRuntime => {
                        println!("     {} {}", Green.paint(""), Green.paint(sd_unit.name))
                    }
                    systemd::UnitState::Masked
                    | systemd::UnitState::MaskedRuntime
                    | systemd::UnitState::Disabled
                    | systemd::UnitState::Bad => {
                        println!("     {} {}", Red.paint(""), Red.paint(sd_unit.name))
                    }
                    _ => println!("     {} {}", Yellow.paint("卑"), Yellow.paint(sd_unit.name)),
                }
            }
        }
        println!("\n - {}", Cyan.bold().paint("Docker Containers"));
        for process in get_docker_processes() {
            println!("     {}", process);
        }
    } else {
        println!("Unable to get terminal size");
    }
}
