use dbus::blocking::Connection;
use std::{path::Path, time::Duration};

pub struct SystemdUnit {
    pub name: String,
    pub state: UnitState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitState {
    Bad,
    Disabled,
    Enabled,
    EnabledRuntime,
    Generated,
    Indirect,
    Linked,
    LinkedRuntime,
    Masked,
    MaskedRuntime,
    Static,
    Transient,
}
impl UnitState {
    /// Takes the string containing the state information from the dbus message and converts it
    /// into a UnitType by matching the first character.
    pub fn new(x: &str) -> UnitState {
        match x {
            "static" => UnitState::Static,
            "disabled" => UnitState::Disabled,
            "enabled" => UnitState::Enabled,
            "enabled-runtime" => UnitState::EnabledRuntime,
            "indirect" => UnitState::Indirect,
            "linked" => UnitState::Linked,
            "linked-runtime" => UnitState::LinkedRuntime,
            "masked" => UnitState::Masked,
            "masked-runtime" => UnitState::MaskedRuntime,
            "bad" => UnitState::Bad,
            "generated" => UnitState::Generated,
            "transient" => UnitState::Transient,
            _ => panic!("Unknown State: {}", x),
        }
    }
}

pub fn list_unit_files() -> Result<Vec<SystemdUnit>, Box<dyn std::error::Error>> {
    let mut systemd = Vec::with_capacity(5);
    // First open up a connection to the session bus.
    let conn = Connection::new_system()?;

    // Second, create a wrapper struct around the connection that makes it easy
    // to send method calls to a specific destination and path.
    let proxy = conn.with_proxy(
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        Duration::from_millis(5000),
    );

    // Now make the method call. The ListNames method call takes zero input parameters and
    // one output parameter which is an array of strings.
    // Therefore the input is a zero tuple "()", and the output is a single tuple "(names,)".
    let (names,): (Vec<(String, String)>,) =
        proxy.method_call("org.freedesktop.systemd1.Manager", "ListUnitFiles", ())?;

    // Let's print all the names to stdout.
    for (a, b) in names {
        systemd.push(SystemdUnit {
            name: String::from(str::replace(
                Path::new(&a)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split('.')
                    .next()
                    .unwrap_or(""),
                '@',
                "",
            )),
            state: UnitState::new(&b),
        })
    }
    Ok(systemd)
}
