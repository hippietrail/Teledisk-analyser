use std::{
    env,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};
use flate2::read::GzDecoder;
use tar::Archive;
use walkdir::WalkDir;
use zip::ZipArchive;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }
    let start_path = &args[1];
    let walkdir = WalkDir::new(start_path).into_iter();
    for dirent in walkdir {
        // iterate, filtering out directories
        let dirent = dirent.expect("Failed to read directory entry");
        if dirent.file_type().is_dir() { continue; }

        // filename tests
        let norm_file_name = dirent.file_name().to_string_lossy().to_lowercase();
        let has_zip_ext = norm_file_name.ends_with("zip");
        let has_gzip_ext = [".tgz", ".gz", ".gzip"].iter().any(|ext| norm_file_name.ends_with(ext));
        let has_tar_ext = norm_file_name.ends_with("tar");

        let mut file = File::open(dirent.path()).expect("Failed to open file");

        // file content tests
        let zip_magic = b"PK\x03\x04";
        let gzip_magic = b"\x1f\x8b";
        // tar doesn't have a magic number

        let mut magic_bytes = [0; 4];
        file.read_exact(&mut magic_bytes).expect("Failed to read file magic");

        let has_zip_magic = &magic_bytes[..4] == zip_magic;
        let has_gzip_magic = &magic_bytes[..2] == gzip_magic;

        // since tar doesn't have a magic number, best check in rust seems to be to instantiate and try the iterator
        // TODO we are currently specifically checking only for a tar inside a gzip!!
        let contains_tar = {
            let mut arc = Archive::new(GzDecoder::new(&file));
            arc.entries().unwrap().next().unwrap().is_ok()
        };

        let file_type = if has_zip_ext || has_zip_magic {
            "Zip"
        } else if (has_gzip_ext || has_gzip_magic) || contains_tar {
            "Tarball"
        } else {
            "File"
        };

        if file_type == "Zip" {
            process_zip_archive(file, &norm_file_name);
        } else if file_type == "Tarball" {
            process_tarball(file, &norm_file_name);
        } else if norm_file_name.to_lowercase().ends_with(".td0") {
            file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of file");
            analyze_teledisk_image_format_from_stream("F", None, &norm_file_name, &mut file);
        }
    }
}

pub fn process_zip_archive(file: File, container_name: &str) {
    let buf_reader = BufReader::new(file);
    let mut archive = ZipArchive::new(buf_reader).expect("Failed to read zip archive");
    for i in 0..archive.len() {
        if let Ok(mut zip_file) = archive.by_index(i) {
            if zip_file.name().to_lowercase().ends_with(".td0") {
                let zip_file_name = zip_file.name().to_string();
                analyze_teledisk_image_format_from_stream("Z", Some(container_name), &zip_file_name, &mut zip_file);
            }
        }
    }
}

pub fn process_tarball(mut file: File, container_name: &str) {
    file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of file");
    let mut archive = Archive::new(GzDecoder::new(file));
    let entries = archive.entries().expect("Failed to read tarball");
    for (i, entry) in entries.enumerate() {
        match entry {
            Ok(mut entry) => {
                if entry.path().unwrap().to_str().unwrap().to_lowercase().ends_with(".td0") {
                    let tar_file_name = entry.path().unwrap().to_string_lossy().to_string();
                    analyze_teledisk_image_format_from_stream("T", Some(container_name), &tar_file_name, &mut entry);
                }
            },
            Err(_err) => {}//println!("T Failed to read tar entry: {} {} at {}", container_name, err, i),
        }
    }
}

#[derive(Debug)]
struct TeleDiskHeaders {
    image_header: ImageHeader,              // Standard header
    comment_header: Option<CommentHeader>,  // Optional comment header
}

impl TeleDiskHeaders {
    fn from_stream(file: &mut dyn Read) -> Self {
        let mut header_bytes = [0; 12];
        file.read_exact(&mut header_bytes).expect("Failed to read image header");
        let image_header = ImageHeader::from_bytes(&header_bytes);

        let mut comment_header = None;

        if image_header.has_comment_header() {
            let mut comment_bytes = [0; 10];
            file.read_exact(&mut comment_bytes).expect("Failed to read comment header");
            comment_header = Some(CommentHeader::from_bytes(&comment_bytes));
        }

        TeleDiskHeaders {
            image_header,
            comment_header,
        }
    }
}

#[derive(Debug)]
struct ImageHeader {
    signature: [u8; 2], // Signature to identify the file format
    sequence: u8,      // Sequence number
    _check_sequence: u8, // Check sequence
    version: u8,       // Version of the disk image format
    data_rate: u8,     // Data rate of the disk image
    drive_type: u8,    // Type of the drive
    stepping: u8,      // Stepping field
    dos_flag: u8,      // DOS allocation flag
    sides: u8,         // Number of sides
    _crc: u16,         // CRC of the header
}

impl ImageHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 12, "ImageHeader must be 12 bytes long");

        let signature = [bytes[0], bytes[1]]; // Extract signature
        let sequence = bytes[2];
        let _check_sequence = bytes[3];
        let version = bytes[4];
        let data_rate = bytes[5];
        let drive_type = bytes[6];
        let stepping = bytes[7];
        let dos_flag = bytes[8];
        let sides = bytes[9];
        let _crc = u16::from_le_bytes([bytes[10], bytes[11]]); // Extract CRC

        ImageHeader {
            signature,
            sequence,
            _check_sequence,
            version,
            data_rate,
            drive_type,
            stepping,
            dos_flag,
            sides,
            _crc,
        }
    }

    // Method to check if a comment header is present
    fn has_comment_header(&self) -> bool {
        self.stepping & 0x80 == 0x80
    }

    // Optionally, you can add a method to validate the signature
    fn is_valid(&self) -> bool {
        self.signature == [0x54, 0x44] // Example signature check
    }
}

#[derive(Debug)]
struct CommentHeader {
    _crc: u16,       // 16-bit CRC of the comment header
    length: u16,     // Length of the comment
    year: u8,        // Year of the comment
    month: u8,       // Month of the comment
    day: u8,         // Day of the comment
    hour: u8,        // Hour of the comment
    minute: u8,      // Minute of the comment
    second: u8,      // Second of the comment
}

impl CommentHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 10, "CommentHeader must be 10 bytes long");

        let _crc = u16::from_le_bytes([bytes[0], bytes[1]]);
        let length = u16::from_le_bytes([bytes[2], bytes[3]]);
        let year = bytes[4];
        let month = bytes[5];
        let day = bytes[6];
        let hour = bytes[7];
        let minute = bytes[8];
        let second = bytes[9];

        CommentHeader {
            _crc,
            length,
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

#[derive(Debug)]
struct TrackHeader {
    number_of_sectors: u8, // Number of sectors in the track
    cylinder_number: u8,    // Cylinder number of the track
    side_number: u8,        // Side number of the track
}

impl TrackHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 4, "TrackHeader must be 4 bytes long");

        let number_of_sectors = bytes[0];
        let cylinder_number = bytes[1];
        let side_number = bytes[2];

        TrackHeader {
            number_of_sectors,
            cylinder_number,
            side_number,
        }
    }
}

#[derive(Debug)]
struct SectorHeader {
    cylinder_number: u8,      // Cylinder number of the sector
    side_number: u8,          // Side number of the sector
    sector_number: u8,        // Sector number
    sector_size: u8,          // Size of the sector
    flags: u8,                // Flags associated with the sector
}

impl SectorHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 6, "SectorHeader must be 6 bytes long");

        let cylinder_number = bytes[0];
        let side_number = bytes[1];
        let sector_number = bytes[2];
        let sector_size = bytes[3];
        let flags = bytes[4];

        SectorHeader {
            cylinder_number,
            side_number,
            sector_number,
            sector_size,
            flags,
        }
    }
}

fn analyze_teledisk_image_format_from_stream(typ: &str, container_name: Option<&str>, file_name: &str, file: &mut dyn Read) {
    let mut parts = Vec::new();
    if let Some(container) = container_name {
        parts.push(container.to_string());
    }
    parts.push(file_name.to_string());
    let joined = parts.join(" / ");

    let headers = TeleDiskHeaders::from_stream(file);

    if headers.image_header.is_valid() {
        let td0_path = if let Some(container) = container_name {
            format!("{} / {}", container, file_name)
        } else {
            file_name.to_string()
        };

        println!("{} : {}{} seq {:02x} ver {:02x} rate {:02x} type {:02x} oh {} step {:02x} dos {:02x} sides {:02x} - {}",
            typ, headers.image_header.signature[0] as char, headers.image_header.signature[1] as char,
            headers.image_header.sequence, headers.image_header.version, headers.image_header.data_rate, headers.image_header.drive_type,
            if headers.image_header.stepping & 0x80 == 0x80 { "O" } else { "-" },
            headers.image_header.stepping & 0x7f, headers.image_header.dos_flag, headers.image_header.sides, td0_path);

        if let Some(comment_header) = headers.comment_header {
            let date = NaiveDate::from_ymd_opt((comment_header.year as i32) + 1900, (comment_header.month as u32) + 1, comment_header.day as u32).unwrap();
            let time = NaiveTime::from_hms_opt(comment_header.hour as u32, comment_header.minute as u32, comment_header.second as u32).unwrap();
            let datetime = NaiveDateTime::new(date, time);
            println!("    {} : {}", datetime, comment_header.length);

            // now we read 'length' bytes which we will convert to an ascii string (it's padded with zeros)
            let mut data = vec![0; comment_header.length as usize];
            file.read_exact(&mut data).expect("Failed to read data");
            let data = String::from_utf8_lossy(&data).to_string();
            println!("    {} : {}", datetime, data);
        }
        analyse_track_and_sector_data(file, typ, headers.image_header, td0_path);
    }
}

fn analyse_track_and_sector_data(file: &mut dyn Read, typ: &str, header: ImageHeader, td0_path: String) {
    for t in 0.. {
        // read 1st track info
        let mut track = [0; 4];
        file.read_exact(&mut track).expect("Failed to read track info");
        let track_header = TrackHeader::from_bytes(&track);

        if track_header.number_of_sectors == 255 { break; }

        for s in 0..track_header.number_of_sectors {
            // read sector info
            let mut sect = [0; 6];
            file.read_exact(&mut sect).expect("Failed to read sector info");
            let sector_header = SectorHeader::from_bytes(&sect);

            if t == 0 && s == 0 {
                println!("{} : {}{} seq {:02x} ver {:02x} rate {:02x} type {:02x} oh {} step {:02x} dos {:02x} sides {:02x} \
                            - [n{} c{:3} h{}] [c{:3} h{} s{} z{} f{:02x}] - {}",
                    typ, header.signature[0] as char, header.signature[1] as char,
                    header.sequence, header.version, header.data_rate, header.drive_type,
                    if header.stepping & 0x80 == 0x80 { "O" } else { "-" },
                    header.stepping & 0x7f, header.dos_flag, header.sides,
                    track_header.number_of_sectors, track_header.cylinder_number, track_header.side_number,
                    sector_header.cylinder_number, sector_header.side_number, sector_header.sector_number, 128 << sector_header.sector_size, sector_header.flags,
                    td0_path
                );
            } else if s == 0 {
                println!("{: ^68}[n{} c{:3} h{}] [c{:3} h{} s{} z{} f{:02x}]", "", track_header.number_of_sectors, track_header.cylinder_number, track_header.side_number, sector_header.cylinder_number, sector_header.side_number, sector_header.sector_number, 128 << sector_header.sector_size, sector_header.flags);
            } else {
                println!("{: ^81}[c{:3} h{} s{} z{} f{:02x}]", "", sector_header.cylinder_number, sector_header.side_number, sector_header.sector_number, 128 << sector_header.sector_size, sector_header.flags);
            }

            // data block
            let mut dblen = [0; 2];
            file.read_exact(&mut dblen).expect("Failed to read data block length");
            let dblen = u16::from_le_bytes(dblen);
            let mut datablock = vec![0; dblen as usize];
            file.read_exact(&mut datablock).expect("Failed to read data block");

            if t == 0 && s == 0 {
                // analyse_tdo_sector(&datablock);
            }
        }
    }

    // see if there are any more bytes
    let mut more = [0; 64];
    let r = file.read(&mut more).expect("Failed to read more");
    if r == 0 {
        println!("EOF");
    } else {
        println!("Read {} more bytes: 0x{:x?}", r, &more[0..r]);
    }
}

fn analyse_tdo_sector(datablock: &[u8]) {
    // TODO
    println!("TDO sector: '{:x?}'", datablock);
}