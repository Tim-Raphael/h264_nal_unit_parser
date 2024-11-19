use std::{fs::File, io::Read};

#[allow(dead_code)]
#[derive(Debug)]
enum NalUnit {
    Slice(Vec<u8>), // video data (I-frame, P-frame, B-frame slice)
    Sps(Vec<u8>),   // sequence parameter set
    Pps(Vec<u8>),   // picture parameter set
    Other(u8, Vec<u8>),
}

fn parse_nal_units(buf: &[u8]) -> Vec<NalUnit> {
    let mut nal_units = Vec::new();
    let mut padding_bytes = 0;
    let mut start_index: Option<usize> = None;

    for i in 0..buf.len() {
        if buf[i] == 0x00 {
            padding_bytes += 1;
            continue;
        } else if buf[i] != 0x01 {
            padding_bytes = 0;
            continue;
        } else if padding_bytes < 2 {
            continue;
        }

        if let Some(start) = start_index {
            let header = &buf[start];

            // a value of 1 indicates that the unit may contain bit errors
            // or other syntax violations
            let forbidden_zero_bit = (header & 0x80) >> 7;

            if forbidden_zero_bit == 1 {
                continue;
            }

            // a value of 00 indicates that the nal unit is not used
            // to reconstructed a image frame
            let nal_ref_idc = (header & 0x60) >> 5;

            let nal_unit_type = header & 0x1F;

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

fn main() -> std::io::Result<()> {
    let mut f = File::open("./output.h264")?;
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);

    let nal_units = parse_nal_units(&buf);

    dbg!(nal_units);

    Ok(())
}
