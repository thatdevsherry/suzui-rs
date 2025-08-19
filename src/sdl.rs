use serialport::TTYPort;
use std::{
    collections::HashMap,
    io::{Read, Write},
    time::{Duration, Instant},
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

use crate::strings::{DISTANCE_FUEL_FILE_PATH, VAG_KKL_PORT};

#[derive(Debug)]
pub struct ScanToolParameterValue {
    pub value: f32,
    pub unit: Option<String>,
}

/// Flow rate for a single injector in (cc/min). Spec for inj. is about 38-48 (cc/15s) i.e 152-192
/// (cc/min).
const INJECTOR_FLOW_RATE: u8 = 152;

/// Struct that contains all processed engine parameters with their representative values.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct EngineContext {
    /// Intended idle by ECU. Affected by A/C idle-up and ECT.
    pub desired_idle: u16,

    /// Current engine RPM.
    pub engine_speed: u16,

    /// ISC flow duty i.e. how much IACV valve is open, expressed as a percentage from 0-100%.
    pub isc_flow_duty: u8,

    /// Throttle position expressed as a percentage of 0-100% of the input voltage to TPS.
    pub absolute_throttle_position: u8,

    /// Calculated throttle angle by ECU.
    pub throttle_angle: u8,

    /// Injector pulse width of injector for cylinder 1. Although not explicitly mentioned, this
    /// usually means the same value will be used for all injectors sequentially and there is no
    /// support for individual injector pulse width.
    pub injector_pulse_width_cyl_1: f32,

    /// Temperature reading by Engine Coolant Temperature (ECT) sensor.
    pub coolant_temp: i8,

    /// Vehicle speed processed by ECU from Vehicle Speed Sensor (VSS).
    pub vehicle_speed: u8,

    /// Temperature reading by Intake Air Temperature (IAT) sensor.
    pub intake_air_temperature: i8,

    /// Reading from MAP sensor in kPa.
    pub manifold_absolute_pressure: f32,

    /// Reading from MAP sensor in kPa, taken just before the first crank. No dedicated sensor.
    pub barometric_pressure: f32,

    /// Battery voltage as read by the ECU. This is not indicative of the actual battery voltage,
    /// just what the ECU is being supplied through the dedicated BATT+ wire.
    pub battery_voltage: f32,

    /// Ignition advance as being commanded by the ECU. Fixed spark is 5 BTDC for verification.
    pub ignition_advance: i8,

    /// Switch to tell if throttle is fully closed. Used to engage idle strategy, fuel cut etc.
    pub closed_throttle_position: bool,

    /// Switch to tell if electric load is active, triggered by defogger & tail-lights.
    pub electric_load: bool,

    /// Custom switch that tells if fuel cut (DFCO) is active.
    pub fuel_cut: bool,

    /// Switch that tells if A/C compressor is active. This is tied to the actual A/C signal that
    /// is sent by the ECU towards the A/C system and is not a reflection of the A/C button.
    pub ac_switch: bool,

    /// Switch to indicate if Power Steering Pump (PSP) switch is closed.
    pub psp_switch: bool,

    /// Switch to indicate if radiator fan is running. I'm not sure if it's just a boolean that
    /// turns ON once the temp threshold goes past or if it is activated when the fan relay is
    /// actually working.
    pub radiator_fan: bool,

    /// Custom calculation related to OBD2 formula that calculates engine load since ECU does not
    /// provide us with it's own value. Hence calculated.
    pub calculated_load: u8,

    /// Instant fuel consumption using fuel flow and speed. Useful only for analyzing driving
    /// habits relation to fuel consumption. Measured in (L/100km).
    pub instant_consumption: f64,

    /// Cumulative distance measured in kilometres (km). This is for long-term fuel consumption
    /// calculation and hence only calculates distance when car engine was running and vehicle
    /// speed was greater than 0.
    pub cumulative_distance: f64,

    /// Cumulative fuel usage measured in litres (L). This is for long-term fuel consumption
    /// calculation and hence only calculates fuel usage when car engine was running and vehicle
    /// speed was greater than 0.
    pub cumulative_fuel: f64,

    /// Total fuel used whenever car engine was running. Measured in litres (L).
    pub total_fuel_used: f64,

    /// Long term fuel consumption based on distance, expressed in (L/100km).
    pub fuel_consumption: f64,

    /// Instantaneous fuel flow rate in (L/hr).
    pub fuel_flow_rate: f64,

    /// Time when ECU was last polled for data.
    pub last_poll: Option<Instant>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Hash, FromRepr, Display)]
pub enum ObdAddress {
    FaultCodes1 = 0x00,
    FaultCodes2 = 0x01,
    FaultCodes3 = 0x02,
    FaultCodes4 = 0x03,
    RpmHigh = 0x04,
    RpmLow = 0x05,
    TargetIdle = 0x06,
    VehicleSpeedSensor = 0x07,
    EngineCoolantTemperature = 0x08,
    IntakeAirTemperature = 0x09,
    TpsAngle = 0x0A,
    TpsVoltage = 0x0B,
    InjectorPulseWidthHigh = 0x0D,
    InjectorPulseWidthLow = 0x0E,
    IgnitionAdvance = 0x0F,
    ManifoldAbsolutePressure = 0x10,
    BarometricPressure = 0x11,
    IscFlowDuty = 0x12,
    BatteryVoltage = 0x16,
    RadiatorFan = 0x19,
    StatusFlags = 0x1a,
    FaultCodes5 = 0x20,
    FaultCodes6 = 0x21,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Hash, Display)]
pub enum ScanToolParameter {
    DesiredIdle,
    EngineSpeed,
    IacFlowDutyCycle,
    AbsoluteThrottlePosition,
    ThrottleAngle,
    InjPulseWidthCyl1,
    CoolantTemp,
    VehicleSpeed,
    IntakeAirTemp,
    Map,
    BarometricPressure,
    BatteryVoltage,
    IgnitionAdvance,
    ClosedThrottlePos,
    ElectricLoad,
    FuelCut,
    AcSwitch,
    PspSwitch,
    RadiatorFan,
    CalculatedLoad,
    FuelConsumption,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromRepr)]
enum SdlHeader {
    Id = 0x10,
    Data = 0x13,
    Actuate = 0x15,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SdlMessage {
    header: SdlHeader,
    length: u8,
    data: Option<Vec<u8>>,
    checksum: u8,
}

impl SdlMessage {
    fn generate_checksum(bytes: &[u8]) -> u8 {
        bytes
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_add(b))
            .wrapping_neg()
    }

    pub fn new(header: SdlHeader, data: Option<Vec<u8>>) -> Self {
        let data = data.unwrap_or_default();
        let length = 1 + 1 + data.len() as u8 + 1;
        let checksum = Self::generate_checksum(&[&[header as u8, length], &data[..]].concat());
        Self {
            header,
            length,
            data: Some(data),
            checksum,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(self.header as u8);
        bytes.push(self.length);
        if let Some(data) = &self.data {
            let data_bytes: Vec<u8> = data.to_vec();
            bytes.extend_from_slice(&data_bytes);
        }
        bytes.push(self.checksum);
        bytes
    }
}

impl TryFrom<&[u8]> for SdlMessage {
    type Error = String;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err("length of bytes less than min. of 3".to_string());
        }
        let header = value[0];
        let header_enum = SdlHeader::from_repr(header.into()).unwrap();
        let length = value[1];
        let data = &value[2..length as usize - 1];
        let d = if !data.is_empty() {
            data.to_vec()
        } else {
            vec![]
        };
        let checksum = value[length as usize - 1];
        Ok(Self {
            header: header_enum,
            length,
            data: Some(d),
            checksum,
        })
    }
}

#[derive(Debug)]
pub struct SuzukiSdlViewer {
    port: Option<TTYPort>,
    pub ecu_id: Option<String>,
    pub raw_data: HashMap<ObdAddress, u8>,
    pub engine_context: EngineContext,
}

impl Default for SuzukiSdlViewer {
    fn default() -> Self {
        let vag_kkl = serialport::new(VAG_KKL_PORT, 7812)
            .timeout(Duration::from_secs(1))
            .open_native();
        let mut raw_data: HashMap<ObdAddress, u8> = HashMap::new();

        for obd_address in ObdAddress::iter() {
            raw_data.insert(obd_address, 0);
        }

        // load up cumulative data from file if valid.
        let mut engine_context = EngineContext::default();
        let distance_fuel =
            std::fs::read_to_string(DISTANCE_FUEL_FILE_PATH).unwrap_or("0,0,0".to_string());
        let split: Vec<&str> = distance_fuel.trim().split(',').collect();

        engine_context.cumulative_distance =
            split.get(0).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        engine_context.cumulative_fuel = split.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
        engine_context.total_fuel_used = split.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);

        Self {
            port: vag_kkl.ok(),
            ecu_id: None,
            raw_data,
            engine_context,
        }
    }
}

impl SuzukiSdlViewer {
    /// Query ECU ID.
    fn get_ecu_id(&mut self) -> String {
        let header = SdlHeader::Id;
        let data = None;
        let sdl_message = SdlMessage::new(header, data);
        let written = self.port.as_mut().unwrap().write(&sdl_message.to_bytes());
        let bytes_written = written.unwrap();
        let mut echo_buf = vec![0; bytes_written];
        let mut response_buf = vec![0; 5];
        self.port
            .as_mut()
            .unwrap()
            .read_exact(&mut echo_buf)
            .unwrap(); // echo
        self.port
            .as_mut()
            .unwrap()
            .read_exact(&mut response_buf)
            .unwrap();
        let ecu_id = &response_buf[2..=3];
        format!("{:#02x}{:#02x}", ecu_id[0], ecu_id[1])
    }

    /// Query obd addresses and update raw data.
    pub fn update_raw_data(&mut self, should_simulate: bool) {
        if should_simulate {
            for (_, v) in self.raw_data.iter_mut() {
                *v = v.wrapping_add(1);
            }
            return;
        }
        let header = SdlHeader::Data;
        let data = Some(
            ObdAddress::iter()
                .filter(|v| {
                    !matches!(
                        v,
                        ObdAddress::FaultCodes1
                            | ObdAddress::FaultCodes2
                            | ObdAddress::FaultCodes3
                            | ObdAddress::FaultCodes4
                            | ObdAddress::FaultCodes5
                            | ObdAddress::FaultCodes6
                    )
                })
                .map(|v| v as u8)
                .collect(),
        );
        let sdl_message = SdlMessage::new(header, data);
        let written = self.port.as_mut().unwrap().write(&sdl_message.to_bytes());
        let bytes_written = written.unwrap();
        let mut echo_buf: Vec<u8> = vec![0; bytes_written];
        let mut response_buf: Vec<u8> = vec![0; bytes_written];
        let _ = self.port.as_mut().unwrap().read_exact(&mut echo_buf); // echo
        let _ = self
            .port
            .as_mut()
            .unwrap()
            .read_exact(response_buf.as_mut_slice());
        let request = sdl_message;
        let response = SdlMessage::try_from(&response_buf[..]).unwrap();

        if let Some(addrs) = request.data
            && let Some(values) = response.data
        {
            for (addr, value) in addrs.iter().zip(values.iter()) {
                if let Some(obd_addr) = ObdAddress::from_repr(*addr as usize) {
                    self.raw_data.insert(obd_addr, *value);
                }
            }
        }
    }

    /// Update scan tool data from raw values.
    pub fn update_processed_data(&mut self) {
        for scan_tool_parameter in ScanToolParameter::iter() {
            match scan_tool_parameter {
                ScanToolParameter::DesiredIdle => {
                    let raw_value = self.raw_data.get(&ObdAddress::TargetIdle).unwrap();
                    self.engine_context.desired_idle = Self::calculate_desired_idle(*raw_value);
                }
                ScanToolParameter::EngineSpeed => {
                    let low_byte = self.raw_data.get(&ObdAddress::RpmLow).unwrap();
                    let high_byte = self.raw_data.get(&ObdAddress::RpmHigh).unwrap();
                    self.engine_context.engine_speed =
                        Self::calculate_rpm_high(*high_byte) + Self::calculate_rpm_low(*low_byte);
                }
                ScanToolParameter::IacFlowDutyCycle => {
                    let raw_value = self.raw_data.get(&ObdAddress::IscFlowDuty).unwrap();
                    self.engine_context.isc_flow_duty = Self::calculate_isc_flow_duty(*raw_value);
                }
                ScanToolParameter::ThrottleAngle => {
                    let raw_value = self.raw_data.get(&ObdAddress::TpsAngle).unwrap();
                    self.engine_context.throttle_angle = Self::calculate_tps_angle(*raw_value);
                }
                ScanToolParameter::BatteryVoltage => {
                    let raw_value = self.raw_data.get(&ObdAddress::BatteryVoltage).unwrap();
                    self.engine_context.battery_voltage =
                        Self::calculate_battery_voltage(*raw_value);
                }
                ScanToolParameter::CoolantTemp | ScanToolParameter::IntakeAirTemp => {
                    let raw_value = if scan_tool_parameter == ScanToolParameter::CoolantTemp {
                        self.raw_data
                            .get(&ObdAddress::EngineCoolantTemperature)
                            .unwrap()
                    } else {
                        self.raw_data
                            .get(&ObdAddress::IntakeAirTemperature)
                            .unwrap()
                    };
                    let processed_value = Self::calculate_temps(*raw_value);
                    if scan_tool_parameter == ScanToolParameter::CoolantTemp {
                        self.engine_context.coolant_temp = processed_value;
                    } else {
                        self.engine_context.intake_air_temperature = processed_value;
                    }
                }
                ScanToolParameter::InjPulseWidthCyl1 => {
                    let low_byte = self
                        .raw_data
                        .get(&ObdAddress::InjectorPulseWidthLow)
                        .unwrap();
                    let high_byte = self
                        .raw_data
                        .get(&ObdAddress::InjectorPulseWidthHigh)
                        .unwrap();
                    self.engine_context.injector_pulse_width_cyl_1 =
                        Self::calculate_inj_pw_high(*high_byte)
                            + Self::calculate_inj_pw_low(*low_byte);
                }
                ScanToolParameter::Map | ScanToolParameter::BarometricPressure => {
                    let raw_value = if scan_tool_parameter == ScanToolParameter::Map {
                        self.raw_data
                            .get(&ObdAddress::ManifoldAbsolutePressure)
                            .unwrap()
                    } else {
                        self.raw_data.get(&ObdAddress::BarometricPressure).unwrap()
                    };
                    let processed_value = Self::calculate_pressure(*raw_value);
                    if scan_tool_parameter == ScanToolParameter::Map {
                        self.engine_context.manifold_absolute_pressure = processed_value;
                    } else {
                        self.engine_context.barometric_pressure = processed_value;
                    }
                }
                ScanToolParameter::AbsoluteThrottlePosition => {
                    let raw_value = self.raw_data.get(&ObdAddress::TpsVoltage).unwrap();
                    let processed_value = (*raw_value as f32 / 255.0) * 100.0;
                    self.engine_context.absolute_throttle_position = processed_value.round() as u8;
                }
                ScanToolParameter::VehicleSpeed => {
                    let raw_value = self.raw_data.get(&ObdAddress::VehicleSpeedSensor).unwrap();
                    let processed_value = *raw_value;
                    self.engine_context.vehicle_speed = processed_value;
                }
                ScanToolParameter::IgnitionAdvance => {
                    let raw_value = self.raw_data.get(&ObdAddress::IgnitionAdvance).unwrap();
                    let processed_value = Self::calculate_ignition_advance(*raw_value);
                    self.engine_context.ignition_advance = processed_value;
                }
                ScanToolParameter::CalculatedLoad => {
                    let iat = self.engine_context.intake_air_temperature;
                    let map = self.engine_context.manifold_absolute_pressure;
                    let baro = self.engine_context.barometric_pressure;
                    let processed_value = (map / baro) * (293.15 / (iat as f32 + 273.15)) * 100.0;
                    self.engine_context.calculated_load = processed_value.round() as u8;
                }
                ScanToolParameter::FuelConsumption => {
                    let now = Instant::now();
                    if let Some(last_poll) = self.engine_context.last_poll {
                        let rpm = self.engine_context.engine_speed as f64;
                        if rpm > 0.0 {
                            let inj_pw = self.engine_context.injector_pulse_width_cyl_1;
                            let vss = self.engine_context.vehicle_speed as f64;

                            // calculate duty cycle
                            let engine_cycle_time = 60_000f64 / rpm;
                            let duty_cycle = (inj_pw as f64) / engine_cycle_time;

                            // calculate fuel flow rate
                            let actual_flow_per_injector = INJECTOR_FLOW_RATE as f64 * duty_cycle;
                            let total_fuel_flow = actual_flow_per_injector * 4.0;
                            let fuel_flow_rate_litres_per_hour = total_fuel_flow * 60.0 / 1000.0;

                            let time_delta = Instant::now().duration_since(last_poll).as_secs_f64();
                            let fuel_flow_litres_per_second =
                                fuel_flow_rate_litres_per_hour / 3600.0; // L/second

                            // accumulate
                            let fuel_this_poll = fuel_flow_litres_per_second * time_delta;
                            let distance_this_poll = vss * (time_delta / 3600.0);

                            let instant_consumption: f64;
                            if vss > 0.0 {
                                instant_consumption =
                                    (fuel_flow_rate_litres_per_hour / vss) * 100.0;
                                self.engine_context.cumulative_distance += distance_this_poll;
                                self.engine_context.cumulative_fuel += fuel_this_poll;
                            } else {
                                instant_consumption = 0.0;
                            }
                            self.engine_context.total_fuel_used += fuel_this_poll;
                            self.engine_context.fuel_flow_rate = fuel_flow_rate_litres_per_hour;
                            self.engine_context.instant_consumption = instant_consumption;
                        }
                    }
                    self.engine_context.fuel_consumption =
                        if self.engine_context.cumulative_distance > 0.0 {
                            (self.engine_context.cumulative_fuel
                                / self.engine_context.cumulative_distance)
                                * 100.0
                        } else {
                            0.0
                        };
                    self.engine_context.last_poll = Some(now);
                }
                ScanToolParameter::PspSwitch => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    self.engine_context.psp_switch = Self::calculate_psp_flag(*raw_value);
                }
                ScanToolParameter::AcSwitch => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    self.engine_context.ac_switch = Self::calculate_ac_flag(*raw_value);
                }
                ScanToolParameter::ClosedThrottlePos => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    self.engine_context.closed_throttle_position =
                        Self::calculate_ctp_flag(*raw_value);
                }
                ScanToolParameter::ElectricLoad => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    self.engine_context.electric_load = Self::calculate_el_flag(*raw_value);
                }
                ScanToolParameter::RadiatorFan => {
                    let raw_value = self.raw_data.get(&ObdAddress::RadiatorFan).unwrap();
                    self.engine_context.radiator_fan = Self::calculate_rad_flag(*raw_value);
                }
                ScanToolParameter::FuelCut => {
                    let low_byte = self
                        .raw_data
                        .get(&ObdAddress::InjectorPulseWidthLow)
                        .unwrap();
                    let high_byte = self
                        .raw_data
                        .get(&ObdAddress::InjectorPulseWidthHigh)
                        .unwrap();
                    let processed_value = *low_byte == 0 && *high_byte == 0;
                    self.engine_context.fuel_cut = processed_value;
                }
            }
        }
    }

    /// Send ID request to ECU as a means of verifying connection.
    pub fn connect(&mut self) {
        let ecu_id = self.get_ecu_id();
        self.ecu_id = Some(ecu_id);
    }

    fn calculate_tps_angle(raw: u8) -> u8 {
        let processed_value = (raw as f32 * 125.0) / 255.0;
        processed_value.round() as u8
    }

    fn calculate_rpm_high(raw: u8) -> u16 {
        let processed_value = (((raw as u16) << 8) | 0u16) as f32 / 5.1;
        processed_value.round() as u16
    }

    fn calculate_rpm_low(raw: u8) -> u16 {
        let processed_value = (((0u16) << 8) | raw as u16) as f32 / 5.1;
        processed_value.round() as u16
    }

    fn calculate_desired_idle(raw: u8) -> u16 {
        let processed_value = raw as f32 * 7.84375;
        processed_value.round() as u16
    }

    fn calculate_temps(raw: u8) -> i8 {
        let processed_value = (raw as f32 * 160.0 / 255.0) - 40.0;
        processed_value as i8
    }

    fn calculate_inj_pw_high(raw: u8) -> f32 {
        (((raw as u16) << 8) | 0u16) as f32 * 0.002
    }

    fn calculate_inj_pw_low(raw: u8) -> f32 {
        (((0u16) << 8) | raw as u16) as f32 * 0.002
    }

    fn calculate_ignition_advance(raw: u8) -> i8 {
        let processed_value = (raw as f32 / 255.0) * (78.0 - (-12.0)) + (-12.0);
        processed_value.round() as i8
    }

    fn calculate_pressure(raw: u8) -> f32 {
        (raw as f32 / 255.0) * (146.63 - (-20.0)) + (-20.0)
    }

    fn calculate_ac_flag(raw: u8) -> bool {
        let processed_value = if raw & (1 << 2) != 0 { 1 } else { 0 };
        processed_value == 1
    }
    fn calculate_ctp_flag(raw: u8) -> bool {
        let processed_value = if raw & (1 << 4) != 0 { 1 } else { 0 };
        processed_value == 1
    }
    fn calculate_psp_flag(raw: u8) -> bool {
        let processed_value = if raw & (1 << 1) != 0 { 1 } else { 0 };
        processed_value == 1
    }
    fn calculate_el_flag(raw: u8) -> bool {
        let processed_value = if raw & (1 << 6) != 0 { 1 } else { 0 };
        processed_value == 1
    }
    fn calculate_rad_flag(raw: u8) -> bool {
        let processed_value = if raw == 128 { 1 } else { 0 };
        processed_value == 1
    }
    fn calculate_isc_flow_duty(raw: u8) -> u8 {
        let processed_value = (raw as f32 / 255.0) * 100.0;
        processed_value.round() as u8
    }
    fn calculate_battery_voltage(raw: u8) -> f32 {
        raw as f32 * 0.0787
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::sdl::SuzukiSdlViewer;

    #[test]
    fn test_rpm_high() {
        let inputs: HashMap<u8, u16> = HashMap::from([
            (0, 0),
            (1, 50),
            (32, 1606),
            (64, 3213),
            (96, 4819),
            (128, 6425),
            (160, 8031),
            (192, 9638),
            (224, 11244),
            (255, 12800),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_rpm_high(key), value)
        }
    }

    #[test]
    fn test_rpm_low() {
        let inputs: HashMap<u8, u16> = HashMap::from([
            (0, 0),
            (1, 0),
            (32, 6),
            (64, 13),
            (96, 19),
            (128, 25),
            (160, 31),
            (192, 38),
            (224, 44),
            (255, 50),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_rpm_low(key), value)
        }
    }

    #[test]
    fn test_desired_idle() {
        let inputs: HashMap<u8, u16> = HashMap::from([
            (0, 0),
            (1, 8),
            (32, 251),
            (64, 502),
            (96, 753),
            (128, 1004),
            (160, 1255),
            (192, 1506),
            (224, 1757),
            (255, 2000),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_desired_idle(key), value)
        }
    }

    #[test]
    fn test_temps() {
        let inputs: HashMap<u8, i8> = HashMap::from([
            (0, -40),
            (1, -39),
            (32, -19),
            (64, 0),
            (96, 20),
            (128, 40),
            (160, 60),
            (192, 80),
            (224, 100),
            (255, 120),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_temps(key), value)
        }
    }

    #[test]
    fn test_tps_angle() {
        let inputs: HashMap<u8, u8> = HashMap::from([
            (0, 0),
            (1, 0),
            (32, 16),
            (64, 31),
            (96, 47),
            (128, 63),
            (160, 78),
            (192, 94),
            (224, 110),
            (255, 125),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_tps_angle(key), value)
        }
    }

    #[test]
    fn test_inj_pw_high() {
        let inputs: HashMap<u8, &str> = HashMap::from([
            (0, "0.000"),
            (1, "0.512"),
            (32, "16.384"),
            (64, "32.768"),
            (96, "49.152"),
            (128, "65.536"),
            (160, "81.920"),
            (192, "98.304"),
            (224, "114.688"),
            (255, "130.560"),
        ]);
        for (key, value) in inputs {
            assert_eq!(
                format!("{:.3}", SuzukiSdlViewer::calculate_inj_pw_high(key)),
                value
            )
        }
    }

    #[test]
    fn test_inj_pw_low() {
        let inputs: HashMap<u8, &str> = HashMap::from([
            (0, "0.000"),
            (1, "0.002"),
            (32, "0.064"),
            (64, "0.128"),
            (96, "0.192"),
            (128, "0.256"),
            (160, "0.320"),
            (192, "0.384"),
            (224, "0.448"),
            (255, "0.510"),
        ]);
        for (key, value) in inputs {
            assert_eq!(
                format!("{:.3}", SuzukiSdlViewer::calculate_inj_pw_low(key)),
                value
            )
        }
    }

    #[test]
    fn test_ignition_advance() {
        let inputs: HashMap<u8, i8> = HashMap::from([
            (0, -12),
            (1, -12),
            (32, -1),
            (64, 11),
            (96, 22),
            (128, 33),
            (160, 44),
            (192, 56),
            (224, 67),
            (255, 78),
        ]);
        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_ignition_advance(key), value)
        }
    }

    #[test]
    fn test_pressure() {
        let inputs: HashMap<u8, i16> = HashMap::from([
            (0, -20),
            (1, -19),
            (32, 1),
            (64, 22),
            (96, 43),
            (128, 64),
            (160, 85),
            (192, 105),
            (224, 126),
            (255, 147),
        ]);
        for (key, value) in inputs {
            assert_eq!(
                SuzukiSdlViewer::calculate_pressure(key).round() as i16,
                value
            )
        }
    }

    #[test]
    fn test_isc_flow_duty() {
        assert_eq!(SuzukiSdlViewer::calculate_isc_flow_duty(0), 0);
        assert_eq!(SuzukiSdlViewer::calculate_isc_flow_duty(128), 50);
        assert_eq!(SuzukiSdlViewer::calculate_isc_flow_duty(255), 100);
    }

    #[test]
    fn test_battery_voltage() {
        assert_eq!(SuzukiSdlViewer::calculate_battery_voltage(0).round(), 0.0);
        assert_eq!(
            SuzukiSdlViewer::calculate_battery_voltage(128).round(),
            10.0
        );
        assert_eq!(
            SuzukiSdlViewer::calculate_battery_voltage(255).round(),
            20.0
        );
    }

    #[test]
    fn test_rad_flag() {
        let inputs: HashMap<u8, bool> = HashMap::from([(0, false), (128, true)]);

        for (key, value) in inputs {
            assert_eq!(SuzukiSdlViewer::calculate_rad_flag(key), value)
        }
    }

    #[test]
    fn test_status_flags() {
        let inputs = vec![1, 2, 4, 8, 16, 32, 64];
        for input in inputs {
            assert_eq!(
                SuzukiSdlViewer::calculate_ctp_flag(input),
                if input == 16 { true } else { false }
            );
            assert_eq!(
                SuzukiSdlViewer::calculate_el_flag(input),
                if input == 64 { true } else { false }
            );
            assert_eq!(
                SuzukiSdlViewer::calculate_ac_flag(input),
                if input == 4 { true } else { false }
            );
            assert_eq!(
                SuzukiSdlViewer::calculate_psp_flag(input),
                if input == 2 { true } else { false }
            );
        }
    }
}
