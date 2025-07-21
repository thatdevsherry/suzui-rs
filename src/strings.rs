#[cfg(target_arch = "aarch64")]
pub const DISTANCE_FUEL_FILE_PATH: &str = "/home/dietpi/distance_fuel";

#[cfg(not(target_arch = "aarch64"))]
pub const DISTANCE_FUEL_FILE_PATH: &str = "/tmp/distance_fuel";

pub const VAG_KKL_PORT: &str = "/dev/ttyUSB0";
