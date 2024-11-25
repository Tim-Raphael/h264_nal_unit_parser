use std::fmt;
use std::io;

#[allow(dead_code)]
pub enum NalUnit {
    Slice(Vec<u8>),     // video data (I-frame, P-frame, B-frame slice)
    Sps(Vec<u8>),       // sequence parameter set
    Pps(Vec<u8>),       // picture parameter set
    Other(u8, Vec<u8>), // unspecified or unknown types
}

impl fmt::Display for NalUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NalUnit::Slice(_) => "Slice".to_string(),
                NalUnit::Sps(_) => "SPS".to_string(),
                NalUnit::Pps(_) => "PPS".to_string(),
                NalUnit::Other(id, _) => format!("Other({})", id),
            }
        )
    }
}

#[derive(Default)]
pub struct NalUnitParser {
    zero_byte_count: usize,
    byte_buf: Vec<u8>,
    pub nal_units: Vec<NalUnit>,
}

// impl Stream for NalUnitParser {
//      type Item = NalUnit
//
// }

impl io::Write for NalUnitParser {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for byte in buf {
            self.byte_buf.push(*byte);

            if byte == &0x00 {
                self.zero_byte_count += 1;
            } else if byte == &0x01 && self.zero_byte_count >= 2 {
                let nal_unit = Self::parse(&self.byte_buf[..3]).unwrap();
                self.nal_units.push(nal_unit);
                self.zero_byte_count = 0;
                self.byte_buf.clear();
            } else {
                self.zero_byte_count = 0;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl NalUnitParser {
    pub fn new() -> Self {
        NalUnitParser::default()
    }

    fn parse(bytes: &[u8]) -> eyre::Result<NalUnit> {
        let header = &bytes[0];

        // A value of 1 indicates that the unit may contain bit errors or other syntax violations.
        let forbidden_zero_bit = (header & 0x80) >> 7;

        // A value of 00 indicates that the NalUnit is not used to reconstruct an image frame.
        let nal_ref_idc = (header & 0x60) >> 5;
        let nal_unit_type = header & 0x1F;

        if forbidden_zero_bit == 1 {
            eyre::bail!("forbidden_zero_bit shall be equal to 0.");
        }

        if nal_ref_idc == 0 && nal_unit_type == 5 {
            eyre::bail!(
                "nal_ref_idc shall not be equal to 0 for NAL units with nal_unit_type equal to 5."
            );
        }

        let nal_unit = match nal_unit_type {
            0 => NalUnit::Other(0, bytes[1..].to_vec()), // Unspecified non-VCL
            1..=5 => NalUnit::Slice(bytes[1..].to_vec()), // Slice
            7 => NalUnit::Sps(bytes[1..].to_vec()),      // Sequence parameter set
            8 => NalUnit::Pps(bytes[1..].to_vec()),      // Picture parameter set
            _ => NalUnit::Other(nal_unit_type, bytes[1..].to_vec()), // Catch-all for unknown types
        };

        eyre::Ok(nal_unit)
    }
}
