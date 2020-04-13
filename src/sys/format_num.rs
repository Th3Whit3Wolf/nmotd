use std::fmt;

pub enum MemUnit {
    B(f64),
    KB(f64),
    KiB(f64),
    MB(f64),
    MiB(f64),
    GB(f64),
    GiB(f64),
}

impl MemUnit {
    fn as_str(&self) -> &'static str {
        match self {
            MemUnit::B(_) => "B",
            MemUnit::KB(_) => "KB",
            MemUnit::KiB(_) => "KiB",
            MemUnit::MB(_) => "MB",
            MemUnit::MiB(_) => "MiB",
            MemUnit::GB(_) => "GB",
            MemUnit::GiB(_) => "GiB",
        }
    }
    fn format(&self) -> String {
        let bytes: f64 = match &self {
            MemUnit::B(bytes) => *bytes,
            MemUnit::KB(bytes) => bytes / 1000 as f64,
            MemUnit::KiB(bytes) => bytes / 1024 as f64,
            MemUnit::MB(bytes) => bytes / 1_000_000 as f64,
            MemUnit::MiB(bytes) => bytes / 1_048_576 as f64,
            MemUnit::GB(bytes) => bytes / 1_000_000_000 as f64,
            MemUnit::GiB(bytes) => bytes / 1_073_741_824 as f64,
        };
        let string = bytes.to_string();
        match bytes as u64 {
            0..=9 => {
                let out = "".to_string() + &string[..1];
                return out;
            }
            10..=99 => {
                let out = "".to_string() + &string[..2] + " ";
                return out;
            }
            100..=999 => {
                let out = "".to_string() + &string[..3];
                return out;
            }
            1_000..=9_999 => {
                let out = "".to_string() + &string[..1] + "," + &string[1..4];
                return out;
            }
            10_000..=99_999 => {
                let out = "".to_string() + &string[..2] + "," + &string[2..5];
                return out;
            }
            100_000..=999_999 => {
                let out = "".to_string() + &string[..3] + "," + &string[3..6];
                return out;
            }
            1_000_000..=9_999_999 => {
                let out = "".to_string() + &string[..1] + "," + &string[1..4] + "," + &string[4..7];
                return out;
            }
            10_000_000..=99_999_999 => {
                let out = "".to_string() + &string[..2] + "," + &string[2..5] + "," + &string[5..8];
                return out;
            }
            100_000_000..=999_999_999 => {
                let out = "".to_string() + &string[..3] + "," + &string[3..6] + "," + &string[6..9];
                return out;
            }
            _ => {
                let out = string;
                return out;
            }
        }
    }
}

impl fmt::Display for MemUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.format(), self.as_str())
    }
}
