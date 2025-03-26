use std::fs;
use std::io::{Read, Write};

use tracing::{error, warn};

use super::Bytes;

pub struct File {
    pub file: fs::File,
}

macro_rules! read_type {
    ($obj:expr, $t:ty) => {
        let mut buf = [0u8; std::mem::size_of::<$t>()];
        if let Err(err) = $obj.file.read_exact(&mut buf) {
            error!("Cannot read from the file: {err}");
            return <$t>::default();
        }
        return <$t>::from_le_bytes(buf);
    };
}

macro_rules! write_type {
    ($obj:expr, $v:expr) => {
        if let Err(err) = $obj.file.write($v.to_le_bytes().as_slice()) {
            error!("Cannot write into a file: {err}");
        }
    };
}

#[luajit_ffi_gen::luajit_ffi]
impl File {
    pub fn exists(path: &str) -> bool {
        match fs::metadata(path) {
            Ok(metadata) => metadata.is_file(),
            Err(err) => {
                warn!("Cannot get '{path}' file metadata: {err}");
                false
            }
        }
    }

    pub fn is_dir(path: &str) -> bool {
        match fs::metadata(path) {
            Ok(metadata) => metadata.is_dir(),
            Err(err) => {
                warn!("Cannot get '{path}' file metadata: {err}");
                false
            }
        }
    }

    #[bind(name = "Create")]
    pub fn new(path: &str) -> Option<Self> {
        match fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
        {
            Ok(file) => Some(Self { file }),
            Err(err) => {
                error!("Cannot create file '{path}': {err}");
                None
            }
        }
    }

    pub fn open(path: &str) -> Option<Self> {
        match fs::File::options().create(true).append(true).open(path) {
            Ok(file) => Some(Self { file }),
            Err(err) => {
                error!("Cannot open file '{path}': {err}");
                None
            }
        }
    }

    pub fn close(&self) -> bool {
        if let Err(err) = self.file.sync_all() {
            error!("Cannot cloase file: {err}");
            return false;
        }
        // TODO: proper close?
        true
    }

    pub fn read_bytes(path: &str) -> Option<Bytes> {
        match fs::read(path) {
            Ok(data) => Some(Bytes::from_vec(data)),
            Err(err) => {
                error!("Cannot read bytes from file '{path}': {err}");
                None
            }
        }
    }

    pub fn read_cstr(path: &str) -> Option<String> {
        match fs::read_to_string(path) {
            Ok(s) => Some(s),
            Err(err) => {
                error!("Cannot read string from file '{path}': {err}");
                None
            }
        }
    }

    pub fn size(path: &str) -> Option<u64> {
        match fs::metadata(path) {
            Ok(metadata) => Some(metadata.len()),
            Err(err) => {
                warn!("Cannot get '{path}' file length: {err}");
                None
            }
        }
    }

    pub fn read(&mut self, data: &mut [u8]) -> Option<usize> {
        match self.file.read(data) {
            Ok(size) => Some(size),
            Err(err) => {
                error!("Cannot read data from file: {err}");
                None
            }
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Option<usize> {
        match self.file.write(data) {
            Ok(size) => Some(size),
            Err(err) => {
                error!("Cannot write data to a file: {err}");
                None
            }
        }
    }

    pub fn write_str(&mut self, data: &str) -> Option<usize> {
        let buffer = data.as_bytes();

        match self.file.write(buffer) {
            Ok(size) => Some(size), // TODO: return Option<usize>
            Err(err) => {
                error!("Cannot write string to a file: {err}");
                None
            }
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        read_type!(self, u8);
    }

    pub fn read_u16(&mut self) -> u16 {
        read_type!(self, u16);
    }

    pub fn read_u32(&mut self) -> u32 {
        read_type!(self, u32);
    }

    pub fn read_u64(&mut self) -> u64 {
        read_type!(self, u64);
    }

    pub fn read_i8(&mut self) -> i8 {
        read_type!(self, i8);
    }

    pub fn read_i16(&mut self) -> i16 {
        read_type!(self, i16);
    }

    pub fn read_i32(&mut self) -> i32 {
        read_type!(self, i32);
    }

    pub fn read_i64(&mut self) -> i64 {
        read_type!(self, i64);
    }

    pub fn read_f32(&mut self) -> f32 {
        read_type!(self, f32);
    }

    pub fn read_f64(&mut self) -> f64 {
        read_type!(self, f64);
    }

    pub fn write_u8(&mut self, value: u8) {
        write_type!(self, value);
    }

    pub fn write_u16(&mut self, value: u16) {
        write_type!(self, value);
    }

    pub fn write_u32(&mut self, value: u32) {
        write_type!(self, value);
    }

    pub fn write_u64(&mut self, value: u64) {
        write_type!(self, value);
    }

    pub fn write_i8(&mut self, value: i8) {
        write_type!(self, value);
    }

    pub fn write_i16(&mut self, value: i16) {
        write_type!(self, value);
    }

    pub fn write_i32(&mut self, value: i32) {
        write_type!(self, value);
    }

    pub fn write_64(&mut self, value: i64) {
        write_type!(self, value);
    }

    pub fn write_f32(&mut self, value: f32) {
        write_type!(self, value);
    }

    pub fn write_f64(&mut self, value: f64) {
        write_type!(self, value);
    }
}
