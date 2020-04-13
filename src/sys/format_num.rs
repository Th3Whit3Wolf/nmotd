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
            MemUnit::KB(bytes) => bytes / 1000_f64,
            MemUnit::KiB(bytes) => bytes / 1024_f64,
            MemUnit::MB(bytes) => bytes / 1_000_000_f64,
            MemUnit::MiB(bytes) => bytes / 1_048_576_f64,
            MemUnit::GB(bytes) => bytes / 1_000_000_000_f64,
            MemUnit::GiB(bytes) => bytes / 1_073_741_824_f64,
        };
        let string = bytes.to_string();
        match bytes as u64 {
            0..=9 => "".to_string() + &string[..1],
            10..=99 => "".to_string() + &string[..2] + " ",
            100..=999 => "".to_string() + &string[..3],
            1_000..=9_999 => "".to_string() + &string[..1] + "," + &string[1..4],
            10_000..=99_999 => "".to_string() + &string[..2] + "," + &string[2..5],
            100_000..=999_999 => "".to_string() + &string[..3] + "," + &string[3..6],
            1_000_000..=9_999_999 => {
                "".to_string() + &string[..1] + "," + &string[1..4] + "," + &string[4..7]
            }
            10_000_000..=99_999_999 => {
                "".to_string() + &string[..2] + "," + &string[2..5] + "," + &string[5..8]
            }
            100_000_000..=999_999_999 => {
                "".to_string() + &string[..3] + "," + &string[3..6] + "," + &string[6..9]
            }
            _ => string,
        }
    }
}

impl fmt::Display for MemUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.format(), self.as_str())
    }
}
