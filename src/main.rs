pub mod sys;
pub mod services;

use sys::{
    cpu_info,
    get_kernel,
    get_all_disks,
    hostname,
    loadavg,
    mem_info,
    MemUnit,
    OsRelease,
    process_by_user,
    uptime
};

use services::{
    list_unit_files,
    get_docker_processes,
    systemd
};

use ansi_term::Colour::{Blue, Cyan, Green, Purple, Red, Yellow};
use terminal_size::{terminal_size, Height, Width};

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
                MemUnit::MiB(
                    (mem.total - mem.free - mem.cached - mem.buffers - mem.sreclaimable) as f64
                        * 1024 as f64
                )
                .to_string()
            ),
            Green
                .bold()
                .paint(MemUnit::MiB(mem.free as f64 * 1024 as f64).to_string()),
            Green
                .bold()
                .paint(MemUnit::MiB(mem.total as f64 * 1024 as f64).to_string())
        );

        println!(" - {}", Cyan.bold().paint("Volumes"));
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
                "     [{}{}]",
                Blue.bold().paint(
                    "=".repeat(
                        ((w as f64 - 15.0)
                            * ((disk.total_space - disk.available_space) as f64
                                / disk.total_space as f64))
                            .ceil() as usize
                    )
                ),
                Purple.paint(
                    "=".repeat(
                        ((w as f64 - 15.0)
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
                        println!("     {} {}", Green.bold().paint(""), Green.bold().paint(sd_unit.name))
                    }
                    systemd::UnitState::Masked
                    | systemd::UnitState::MaskedRuntime
                    | systemd::UnitState::Disabled
                    | systemd::UnitState::Bad => {
                        println!("     {} {}", Red.bold().paint(""), Red.bold().paint(sd_unit.name))
                    }
                    _ => println!("     {} {}", Yellow.bold().paint("卑"), Yellow.bold().paint(sd_unit.name)),
                }
            }
        }
        
        match get_docker_processes() {
            Some(processes) => {
                println!("\n - {}", Cyan.bold().paint("Docker Containers"));
                for process in processes {
                    println!("     {}", process);
                }
            },
            None => {}
        }
        
    } else {
        println!("Unable to get terminal size");
    }
}
