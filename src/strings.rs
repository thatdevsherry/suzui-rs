// you will LOL at the path but it's the simplest solution.
// I've set root as read-only and instead of trying to setup a certain dir
// as writeable, saving to `/boot` is easiest as that is on another
// partition and is still writeable even if "ro" flag is set.
#[cfg(target_arch = "aarch64")]
pub const DISTANCE_FUEL_FILE_PATH: &str = "/boot/distance_fuel";

#[cfg(not(target_arch = "aarch64"))]
pub const DISTANCE_FUEL_FILE_PATH: &str = "/tmp/distance_fuel";
