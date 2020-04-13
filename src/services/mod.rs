pub mod docker;
pub mod systemd;

pub use self::docker::get_docker_processes;
pub use self::systemd::list_unit_files;
pub use self::systemd::SystemdUnit;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_list_unit_files() {
        let unit_files = list_unit_files().unwrap();
        assert!(unit_files.len() > 0, true);
        for unit in unit_files {
            println!(
                "Systemd Unit \n\tName: {}\n\tStatus: {:?}",
                unit.name, unit.state
            )
        }
    }
}
