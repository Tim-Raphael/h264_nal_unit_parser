use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum NalUnitType {
    Slice,
    Sps,
    Pps,
    Other(u8),
}

pub struct NalUnitCount {
    count: HashMap<NalUnitType, i32>,
}

impl NalUnitCount {
    pub fn new() -> Self {
        NalUnitCount {
            count: HashMap::new(),
        }
    }

    pub fn update(&mut self, nal_unit: &NalUnit) {
        let key = nal_unit.to_type();
        *self.count.entry(key).or_insert(0) += 1;
    }

    pub fn format(&self) -> String {
        self.count
            .clone()
            .into_iter()
            .map(|(key, count)| format!("{:?}: {}", key, count))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug)]
pub enum NalUnit {
    Slice(Vec<u8>), // video data (I-frame, P-frame, B-frame slice)
    Sps(Vec<u8>),   // sequence parameter set
    Pps(Vec<u8>),   // picture parameter set
    Other(u8, Vec<u8>),
}

impl NalUnit {
    pub fn to_type(&self) -> NalUnitType {
        match self {
            NalUnit::Slice(_) => NalUnitType::Slice,
            NalUnit::Sps(_) => NalUnitType::Sps,
            NalUnit::Pps(_) => NalUnitType::Pps,
            NalUnit::Other(t, _) => NalUnitType::Other(*t),
        }
    }
}

struct NalUnitParser {
    zero_byte_count: usize,
    byte_buf: Vec<u8>,
    nal_units: Vec<NalUnit>,
}

impl NalUnitParser {
    fn parse(&mut self, bytes: &[u8]) -> Option<NalUnit> {
        let header = &bytes[0];

        // a value of 1 indicates that the unit may contain bit errors or other syntax
        // violations
        let forbidden_zero_bit = (header & 0x80) >> 7;

        // a value of 00 indicates that the nal unit is not used to reconstructed a image frame
        let _nal_ref_idc = (header & 0x60) >> 5;
        let nal_unit_type = header & 0x1F;

        if forbidden_zero_bit == 1 {
            return None;
        }

        let nal_unit = match nal_unit_type {
            1..=5 => NalUnit::Slice(bytes[1..].to_vec()),
            7 => NalUnit::Sps(bytes[1..].to_vec()),
            8 => NalUnit::Pps(bytes[1..].to_vec()),
            _ => NalUnit::Other(nal_unit_type, bytes[1..].to_vec()),
        };

        Some(nal_unit)
    }
}

impl Write for NalUnitParser {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for byte in buf {
            self.byte_buf.push(*byte);

            if byte == &0x00 {
                self.zero_byte_count += 1;
            } else if byte != &0x01 || self.zero_byte_count < 2 {
                self.zero_byte_count = 0;
            }

            if byte == &0x01 && self.zero_byte_count >= 2 {
                let nal_bytes = &self.byte_buf[..3].to_vec();

                if let Some(nal_unit) = self.parse(nal_bytes) {
                    self.nal_units.push(nal_unit);
                }

                self.byte_buf.clear();
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn parse(buf: &[u8]) -> Vec<NalUnit> {
    let mut nal_units = Vec::new();
    let mut padding_bytes = 0;
    let mut start_index: Option<usize> = None;

    // struct NalUnitParser {
    //      zero_byte_count: usize,
    //      byte_buf: Vec<u8>,
    //      nal_units: Vec<NalUnit>,
    // }
    //
    // impl Write for NalUnitParser {
    //      fn write(mut self, byte_slice: &[u8]) -> io::Result bla {
    //          for byte in byte_slice {
    //              self.byte_buf.push(byte);
    //
    //              // header detection logic
    //              if byte == 0x00 {
    //                  self.zero_byte_count += 1;
    //              } else if byte != 0x01 || self.zero_byte_count < 2 {
    //                  self.zero_byte_count = 0;
    //              }
    //
    //              if byte == 0x01 && self.zero_byte_count == 2 {
    //                  // this is my nal unit
    //                  let nal_bytes = self.bytes_buf[..3].clone();
    //                  self.nal_units.push( todo!() );
    //                  self.bytes_buf.clear();
    //              }
    //          }
    //      }
    // }
    //
    // impl Stream for NalUnitParser {
    //  type Item = NalUnit
    //
    // }

    for i in 0..buf.len() {
        if buf[i] == 0x00 {
            padding_bytes += 1;
            continue;
        } else if buf[i] != 0x01 || padding_bytes < 2 {
            padding_bytes = 0;
            continue;
        }

        if let Some(start) = start_index {
            let header = &buf[start];

            // a value of 1 indicates that the unit may contain bit errors or other syntax
            // violations
            let forbidden_zero_bit = (header & 0x80) >> 7;

            // a value of 00 indicates that the nal unit is not used to reconstructed a image frame
            let _nal_ref_idc = (header & 0x60) >> 5;
            let nal_unit_type = header & 0x1F;

            if forbidden_zero_bit == 1 {
                continue;
            }

            let nal_unit = match nal_unit_type {
                1..=5 => NalUnit::Slice(buf[(start + 1)..(i - padding_bytes)].to_vec()),
                7 => NalUnit::Sps(buf[(start + 1)..(i - padding_bytes)].to_vec()),
                8 => NalUnit::Pps(buf[(start + 1)..(i - padding_bytes)].to_vec()),
                _ => NalUnit::Other(
                    nal_unit_type,
                    buf[(start + 1)..(i - padding_bytes)].to_vec(),
                ),
            };

            nal_units.push(nal_unit);
        }

        start_index = Some(i + 1);
        padding_bytes = 0;
    }

    nal_units
}
