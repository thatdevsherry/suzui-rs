use rand::{rngs, seq::IndexedRandom, thread_rng};
use serialport::TTYPort;
use std::{
    collections::HashMap,
    io::{Read, Write},
    thread::sleep,
    time::Duration,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, FromRepr};

#[derive(Debug)]
pub struct ScanToolParameterValue {
    pub value: f32,
    pub unit: Option<String>,
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
    InjPulseWidthCyl1,
    CoolantTemp,
    VehicleSpeed,
    IntakeAirTemp,
    Map,
    BarometricPressure,
    TpSensorVolt,
    BatteryVoltage,
    IgnitionAdvance,
    ClosedThrottlePos,
    ElectricLoad,
    FuelCut,
    AcSwitch,
    PspSwitch,
    RadiatorFan,
}

enum Actuate {
    None = 0x00,
    FixedSpark = 0x10,
    Isc = 0xc0,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromRepr)]
enum SdlHeader {
    EcuId = 0x10,
    EcuData = 0x13,
    EcuActuate = 0x15,
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
        let length = header as u8 + data.len() as u8 + 1;
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
            let data_bytes: Vec<u8> = data.iter().map(|&v| v as u8).collect();
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
        let d = if data.len() > 0 {
            data.to_owned().into_iter().collect()
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
    port: TTYPort,
    pub raw_data: HashMap<ObdAddress, u8>,
    pub scan_tool_data: HashMap<ScanToolParameter, ScanToolParameterValue>,
}

impl Default for SuzukiSdlViewer {
    fn default() -> Self {
        let vag_kkl = serialport::new("/dev/ttyUSB0", 7812)
            .timeout(Duration::from_secs(1))
            .open_native()
            .expect("Failed to open port");
        let mut scan_tool_data: HashMap<ScanToolParameter, ScanToolParameterValue> = HashMap::new();
        let mut raw_data: HashMap<ObdAddress, u8> = HashMap::new();

        for obd_address in ObdAddress::iter() {
            raw_data.insert(obd_address, 0);
        }
        for scan_tool_parameter in ScanToolParameter::iter() {
            scan_tool_data.insert(
                scan_tool_parameter,
                ScanToolParameterValue {
                    value: 0.0,
                    unit: None,
                },
            );
        }
        Self {
            port: vag_kkl,
            scan_tool_data,
            raw_data,
        }
    }
}

impl SuzukiSdlViewer {
    /// Query ECU ID.
    fn get_ecu_id(&mut self) -> String {
        let header = SdlHeader::EcuId;
        let data = None;
        let sdl_message = SdlMessage::new(header, data);
        let written = self.port.write(&sdl_message.to_bytes());
        println!("wrote {:?} bytes", written);
        let bytes_written = written.unwrap();
        let mut response_buf = vec![0; bytes_written];
        let bytes_read_echo = self.port.read(response_buf.as_mut_slice()); // echo
        println!("read echo: {:?} bytes", bytes_read_echo);
        let bytes_read = self.port.read(response_buf.as_mut_slice());
        println!("read response: {:?} bytes", bytes_read);
        "FFFF".to_string()
    }

    /// Query obd addresses and update raw data.
    pub fn update_raw_data(&mut self) {
        for (k, v) in self.raw_data.iter_mut() {
            *v = v.wrapping_add(1);
        }
        return;
        let header = SdlHeader::EcuData;
        let data = Some(ObdAddress::iter().map(|v| v as u8).collect());
        let sdl_message = SdlMessage::new(header, data);
        let written = self.port.write(&sdl_message.to_bytes());
        println!("wrote {:?} bytes", written);
        let bytes_written = written.unwrap();
        let mut response_buf = vec![0; bytes_written];
        let bytes_read_echo = self.port.read(response_buf.as_mut_slice()); // echo
        println!("read echo: {:?} bytes", bytes_read_echo);
        let bytes_read = self.port.read(response_buf.as_mut_slice());
        println!("read response: {:?} bytes", bytes_read);
        let request = sdl_message;
        println!("{:?}", response_buf);
        let response = SdlMessage::try_from(&response_buf[..]).unwrap();

        if let Some(addrs) = request.data {
            if let Some(values) = response.data {
                for (addr, value) in addrs.iter().zip(values.iter()) {
                    if let Some(obd_addr) = ObdAddress::from_repr(*addr as usize) {
                        self.raw_data.insert(obd_addr, *value);
                    }
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
                    let processed_value = *raw_value as f32 * 7.84375;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("RPM".to_string()),
                        },
                    );
                }
                ScanToolParameter::EngineSpeed => {
                    let low_byte = self.raw_data.get(&ObdAddress::RpmLow).unwrap();
                    let high_byte = self.raw_data.get(&ObdAddress::RpmHigh).unwrap();
                    let processed_value =
                        (((*high_byte as u16) << 8) | *low_byte as u16) as f32 / 5.1;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("RPM".to_string()),
                        },
                    );
                }
                ScanToolParameter::IacFlowDutyCycle => {
                    let raw_value = self.raw_data.get(&ObdAddress::IscFlowDuty).unwrap();
                    let processed_value = (*raw_value as f32 / 255.0) * 100.0;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("RPM".to_string()),
                        },
                    );
                }
                ScanToolParameter::TpSensorVolt => {
                    let raw_value = self.raw_data.get(&ObdAddress::TpsVoltage).unwrap();
                    let processed_value = (*raw_value as f32 / 255.0) * 5.0;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value,
                            unit: Some("V".to_string()),
                        },
                    );
                }
                ScanToolParameter::BatteryVoltage => {
                    let raw_value = self.raw_data.get(&ObdAddress::BatteryVoltage).unwrap();
                    let processed_value = *raw_value as f32 * 0.0787;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value,
                            unit: Some("V".to_string()),
                        },
                    );
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
                    let processed_value = (*raw_value as f32 / 255.0) * 159.0 - 40.0;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("C".to_string()),
                        },
                    );
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
                    let processed_value =
                        (((*high_byte as u16) << 8) | *low_byte as u16) as f32 * 0.002;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value,
                            unit: Some("ms".to_string()),
                        },
                    );
                }
                ScanToolParameter::Map | ScanToolParameter::BarometricPressure => {
                    let raw_value = if scan_tool_parameter == ScanToolParameter::Map {
                        self.raw_data
                            .get(&ObdAddress::ManifoldAbsolutePressure)
                            .unwrap()
                    } else {
                        self.raw_data.get(&ObdAddress::BarometricPressure).unwrap()
                    };
                    let processed_value =
                        (*raw_value as f32 / 255.0) * (146.63 - (-20.0)) + (-20.0);
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value,
                            unit: Some("kPa".to_string()),
                        },
                    );
                }
                ScanToolParameter::AbsoluteThrottlePosition => {
                    let raw_value = self.raw_data.get(&ObdAddress::TpsVoltage).unwrap();
                    let processed_value = (*raw_value as f32 / 255.0) * 100.0;
                    let abs_throttle_pos = self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("%".to_string()),
                        },
                    );
                }
                ScanToolParameter::VehicleSpeed => {
                    let raw_value = self.raw_data.get(&ObdAddress::VehicleSpeedSensor).unwrap();
                    let processed_value = *raw_value;
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.into(),
                            unit: Some("km/h".to_string()),
                        },
                    );
                }
                ScanToolParameter::IgnitionAdvance => {
                    let raw_value = self.raw_data.get(&ObdAddress::IgnitionAdvance).unwrap();
                    let processed_value = (*raw_value as f32 / 255.0) * (78.0 - (-12.0)) + (-12.0);
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value.round(),
                            unit: Some("BTDC".to_string()),
                        },
                    );
                }
                ScanToolParameter::PspSwitch => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    let processed_value = if raw_value & (1 << 1) != 0 { 1 } else { 0 };
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
                }
                ScanToolParameter::AcSwitch => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    let processed_value = if raw_value & (1 << 2) != 0 { 1 } else { 0 };
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
                }
                ScanToolParameter::ClosedThrottlePos => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    let processed_value = if raw_value & (1 << 4) != 0 { 1 } else { 0 };
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
                }
                ScanToolParameter::ElectricLoad => {
                    let raw_value = self.raw_data.get(&ObdAddress::StatusFlags).unwrap();
                    let processed_value = if raw_value & (1 << 6) != 0 { 1 } else { 0 };
                    self.scan_tool_data.insert(
                        scan_tool_parameter,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
                }
                ScanToolParameter::RadiatorFan => {
                    let raw_value = self.raw_data.get(&ObdAddress::RadiatorFan).unwrap();
                    let processed_value = if *raw_value == 128 { 1 } else { 0 };
                    self.scan_tool_data.insert(
                        ScanToolParameter::RadiatorFan,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
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
                    let processed_value = if *low_byte == 0 && *high_byte == 0 {
                        1
                    } else {
                        0
                    };
                    self.scan_tool_data.insert(
                        ScanToolParameter::FuelCut,
                        ScanToolParameterValue {
                            value: processed_value as f32,
                            unit: None,
                        },
                    );
                }
            }
        }
    }

    /// Send ID request to ECU as a means of verifying connection.
    pub fn connect(&mut self) {
        let ecu_id = self.get_ecu_id();
        println!("ecu_id: {}", ecu_id);
    }
}
