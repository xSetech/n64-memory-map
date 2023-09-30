// SPDX-License-Identifier: GPL-3.0-or-later

//! CLI to describe an N64 virtual address
//!
//! Based on information the memory map documentation here:
//! https://n64brew.dev/wiki/Memory_map
//!
//! The CLI accepts one argument with the following behaviors respectively:
//!
//! 1. If given a 32-bit hexadecimal number prefixed with "0x", details about
//!    the address are printed to stdout.
//!
//! 2. If given a filename of an Ares instruction trace, the virtual address
//!    column is annotated with a short string describing the address.
//!

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::exit;

use regex::Regex;

type Region = (
    u32,            // start
    u32,            // end
    &'static str,   // short name
    &'static str,   // long name
);

static SEGMENTS: &[Region] = &[
    (0x00000000, 0x7FFFFFFF, "U", "KUSEG"),
    (0x80000000, 0x9FFFFFFF, "0", "KSEG0"),
    (0xA0000000, 0xBFFFFFFF, "1", "KSEG1"),
    (0xC0000000, 0xDFFFFFFF, "S", "KSSEG"),
    (0xE0000000, 0xFFFFFFFF, "3", "KSEG3"),
];

static REGIONS: &[Region] = &[
    (0x00000000, 0x03FFFFFF, "R", "RDRAM"),
    (0x04000000, 0x049FFFFF, "G", "RCP"),
    (0x05000000, 0x1FBFFFFF, "P", "PI 1/2"),
    (0x1FC00000, 0x1FCFFFFF, "S", "SI"),
    (0x1FD00000, 0x7FFFFFFF, "B", "PI 2/2"),
    (0x80000000, 0xFFFFFFFF, "U", "Unmapped"),
];

static SUBREGIONS: &[Region] = &[

    // RDRAM (RDR)
    (0x00000000, 0x03EFFFFF, "RDRM", "RDRAM memory-space"),
    (0x03F00000, 0x03F7FFFF, "RDRR", "RDRAM registers"),
    (0x03F80000, 0x03FFFFFF, "RDRB", "RDRAM broadcast registers"),

    // RCP (RSP or RCP)
    (0x04000000, 0x04000FFF, "RSPD", "RSP Data Memory"),
    (0x04001000, 0x04001FFF, "RSPI", "RSP Instruction Memory"),
    (0x04002000, 0x0403FFFF, "RSPM", "RSP DMEM/IMEM Mirrors"),
    (0x04040000, 0x040BFFFF, "RSPR", "RSP Registers"),
    (0x040C0000, 0x040FFFFF, "RCPU", "Unmapped/fatal"),
    (0x04100000, 0x041FFFFF, "RDPC", "RDP Command Registers"),
    (0x04200000, 0x042FFFFF, "RDPS", "RDP Span Registers"),
    (0x04300000, 0x043FFFFF, "InMI", "MIPS Interface"),
    (0x04400000, 0x044FFFFF, "InVI", "Video Interface"),
    (0x04500000, 0x045FFFFF, "InAI", "Audio Interface"),
    (0x04600000, 0x046FFFFF, "InPI", "Peripheral Interface"),
    (0x04700000, 0x047FFFFF, "InRI", "RDRAM Interface"),
    (0x04800000, 0x048FFFFF, "InSI", "Serial Interface"),
    (0x04900000, 0x04FFFFFF, "RCPu", "Unmapped/fatal"),

    // PI
    (0x05000000, 0x05FFFFFF, "NDDR", "N64DD Registers"),
    (0x06000000, 0x07FFFFFF, "NDDI", "N64DD IPL ROM"),
    (0x08000000, 0x0FFFFFFF, "CSRM", "Cartridge SRAM"),
    (0x10000000, 0x1FBFFFFF, "CROM", "Cartridge ROM"),

    // SI
    (0x1FC00000, 0x1FC007BF, "PIFR", "PIF ROM"),
    (0x1FC007C0, 0x1FC007FF, "PIFR", "PIF RAM"),
    (0x1FC00800, 0x1FCFFFFF, "RSVD", "Reserved"),

    // PI, pt.2
    (0x1FD00000, 0x1FFFFFFF, "UPB1", "Unused / PI BUS Domain 1"),
    (0x20000000, 0x7FFFFFFF, "UCPA", "Unused / PI BUS Domain 1 [CPU Accessible]"),

    // No device
    (0x80000000, 0xFFFFFFFF, "UNMP", "Unmapped/fatal"),

];

/// Describes the location of the address by naming its segment, region, and
/// subregion as documented in the mappings above.
#[derive(Debug)]
#[allow(dead_code)]
struct AddressLocation {
    virtual_address: u32,
    physical_address: u32,
    segment: Option<(&'static str, &'static str)>,
    region: Option<(&'static str, &'static str)>,
    subregions: Vec<(&'static str, &'static str)>,
}

/// Given an address, return the name of the segment, region, and subregion
/// where the address is located.
fn get_segment_region_subregion(address: u32) -> AddressLocation {

    // Remove bits about cached/uncached access
    let address_raw: u32 = address & 0x1FFF_FFFF;

    let segment: Option<(&str, &str)> = SEGMENTS.iter()
        .find(|seg| seg.0 <= address && address <= seg.1)
        .map(|seg| (seg.2, seg.3));

    let region: Option<(&str, &str)> = REGIONS.iter()
        .find(|reg| reg.0 <= address_raw && address_raw <= reg.1)
        .map(|reg| (reg.2, reg.3));

    let subregions: Vec<(&str, &str)> = SUBREGIONS.iter()
        .filter(|reg| reg.0 <= address_raw && address_raw <= reg.1)
        .map(|reg| (reg.2, reg.3))
        .collect();

    AddressLocation {
        virtual_address: address,
        physical_address: address_raw,
        segment,
        region,
        subregions,
    }
}

/// Produces the short-form description of an address. The short form is meant
/// to fit into a tight column width.
fn address_location_to_string(address_location: &AddressLocation) -> String {
    let subregion_short_names: Vec<&'static str> = address_location.subregions.iter().map(|s| s.0).collect();
    return format!(
        "{}{}.{}",
        address_location.segment.unwrap_or(("?", "?")).0,
        address_location.region.unwrap_or(("?", "?")).0,
        subregion_short_names.join("."),
    );
}

/// Read a file line by line and apply a regex to each line looking for lines
/// that start with three characters, followed by a space, then 16 hexadecimal
/// characters, and then the rest of the line. Lines that don't match the
/// pattern are printed to stdout as they are. Matching lines are modified so
/// that the hexadecimal part is converted to an integer (u64), and the lower
/// 32 bits are also extracted (u32), and then the modified line is printed
/// to stdout.
fn rewrite_lines_of_file(filename: String) -> io::Result<()> {
    let path: &Path = Path::new(&filename);
    let file: File = File::open(&path)?;
    let reader: io::BufReader<File> = io::BufReader::new(file);
    let re: Regex = Regex::new(r"^([A-Z]{3})\s*([a-f0-9]{16})\s*(.*)$").unwrap();

    for line in reader.lines() {
        let line: String = line?;
        match re.captures(&line) {
            Some(caps) => {

                let prefix: &str = caps.get(1).map_or("", |m| m.as_str());
                let hex: &str = caps.get(2).map_or("", |m| m.as_str());
                let suffix: &str = caps.get(3).map_or("", |m| m.as_str());
                let int_val: u64 = u64::from_str_radix(hex, 16).unwrap();
                let lower_32_bits_val: u32 = (int_val & 0x0000_ffff_ffff) as u32;

                let location: AddressLocation = get_segment_region_subregion(lower_32_bits_val);

                println!(
                    "{} {:<12} {:#08x} {}",
                    prefix,
                    address_location_to_string(&location).to_uppercase(),
                    lower_32_bits_val,
                    suffix
                );
            }
            None => println!("{}", line),
        }
    }

    Ok(())
}

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Expected a file name or an address as argument");
        exit(1);
    }

    let arg = &args[1];
    if arg.starts_with("0x") {
        // Argument is considered an address
        if let Ok(address) = u32::from_str_radix(&arg[2..], 16) {
            let location = get_segment_region_subregion(address);
            println!("{:#?}", location); // Pretty print the AddressLocation struct
        } else {
            eprintln!("Invalid address: {}", arg);
            exit(1);
        }
    } else {
        // Argument is considered a filename
        if let Err(e) = rewrite_lines_of_file(arg.clone()) {
            eprintln!("Error rewriting lines of file {}: {}", arg, e);
            exit(1);
        }
    }

}
