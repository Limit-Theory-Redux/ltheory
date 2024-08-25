use std::io::{Cursor, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;

use super::*;

pub struct Bytes {
    cursor: Cursor<Vec<u8>>,
}

impl Bytes {
    pub fn from_vec(data: Vec<u8>) -> Bytes {
        Bytes {
            cursor: Cursor::new(data),
        }
    }

    /// WARNING: This only works if T is plain-old-data i.e. has no pointers!
    pub fn read<T>(&mut self, data: &mut [T]) {
        self.read_bytes(unsafe { std::mem::transmute::<&mut [T], &mut [u8]>(data) });
    }

    /// WARNING: This only works if T is plain-old-data i.e. has no pointers!
    pub fn write<T>(&mut self, data: &[T]) {
        self.write_bytes(unsafe { std::mem::transmute::<&[T], &[u8]>(data) });
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.cursor.get_mut().as_mut_ptr()
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.cursor.get_ref().as_ptr()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.cursor.get_mut().as_mut_slice()
    }

    pub fn as_slice(&self) -> &[u8] {
        self.cursor.get_ref().as_slice()
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Bytes {
    #[bind(name = "Create")]
    pub fn new(size: u32) -> Bytes {
        Bytes {
            cursor: Cursor::new(vec![0; size as usize]),
        }
    }

    #[bind(name = "CreateWithCapacity")]
    pub fn new_with_capacity(capacity: usize) -> Bytes {
        Bytes {
            cursor: Cursor::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn from_data(data: &[u8]) -> Bytes {
        Self::from_vec(data.to_vec())
    }

    pub fn load(path: &str) -> Bytes {
        let c_path = std::ffi::CString::new(path).unwrap();
        *File_ReadBytes(c_path.as_ptr())
            .unwrap_or_else(|| panic!("Bytes::load: Failed to read file '{path}'"))
    }

    #[bind(name = "GetSize")]
    pub fn len(&self) -> u32 {
        self.cursor.get_ref().len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.cursor.get_ref().is_empty()
    }

    // TODO: Use Result<T>
    pub fn compress(&self) -> Bytes {
        let mut encoder = ZlibEncoder::new(
            Vec::with_capacity(self.len() as usize),
            Compression::default(),
        );
        if let Err(e) = encoder.write_all(self.cursor.get_ref().as_slice()) {
            panic!("Bytes::compress: Encoding failed: {e}");
        }
        Bytes::from_vec(encoder.finish().unwrap())
    }

    // TODO: Use Result<T>
    pub fn decompress(&self) -> Bytes {
        let mut decoder = ZlibDecoder::new(Vec::with_capacity(self.len() as usize));
        if let Err(e) = decoder.write_all(self.cursor.get_ref().as_slice()) {
            panic!("Bytes::compress: Encoding failed: {e}");
        }
        Bytes::from_vec(decoder.finish().unwrap())
    }

    pub fn save(&self, path: &str) {
        let c_path = std::ffi::CString::new(path).unwrap();
        let mut file = File_Create(c_path.as_ptr())
            .unwrap_or_else(|| panic!("Bytes_Save: Failed to open file '{path}' for writing"));
        let _ = file.file.write_all(self.cursor.get_ref().as_slice());
    }

    pub fn get_cursor(&self) -> u32 {
        self.cursor.position() as u32
    }

    pub fn rewind(&mut self) {
        self.cursor.set_position(0);
    }

    pub fn set_cursor(&mut self, cursor: u32) {
        self.cursor.set_position(cursor as u64);
    }

    #[bind(name = "Read")]
    pub fn read_bytes(&mut self, data: &mut [u8]) {
        let _ = self.cursor.read_exact(data);
    }

    pub fn read_u8(&mut self) -> u8 {
        self.cursor.read_u8().unwrap_or_default()
    }

    pub fn read_u16(&mut self) -> u16 {
        self.cursor.read_u16::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_u32(&mut self) -> u32 {
        self.cursor.read_u32::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_u64(&mut self) -> u64 {
        self.cursor.read_u64::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_i8(&mut self) -> i8 {
        self.cursor.read_i8().unwrap_or_default()
    }

    pub fn read_i16(&mut self) -> i16 {
        self.cursor.read_i16::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_i32(&mut self) -> i32 {
        self.cursor.read_i32::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_i64(&mut self) -> i64 {
        self.cursor.read_i64::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_f32(&mut self) -> f32 {
        self.cursor.read_f32::<LittleEndian>().unwrap_or_default()
    }

    pub fn read_f64(&mut self) -> f64 {
        self.cursor.read_f64::<LittleEndian>().unwrap_or_default()
    }

    #[bind(name = "Write")]
    pub fn write_bytes(&mut self, data: &[u8]) {
        let _ = self.cursor.write_all(data);
    }

    pub fn write_str(&mut self, data: &str) {
        self.write(data.as_bytes());
    }

    pub fn write_u8(&mut self, value: u8) {
        let _ = self.cursor.write_u8(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        let _ = self.cursor.write_u16::<LittleEndian>(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        let _ = self.cursor.write_u32::<LittleEndian>(value);
    }

    pub fn write_u64(&mut self, value: u64) {
        let _ = self.cursor.write_u64::<LittleEndian>(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        let _ = self.cursor.write_i8(value);
    }

    pub fn write_i16(&mut self, value: i16) {
        let _ = self.cursor.write_i16::<LittleEndian>(value);
    }

    pub fn write_i32(&mut self, value: i32) {
        let _ = self.cursor.write_i32::<LittleEndian>(value);
    }

    pub fn write_i64(&mut self, value: i64) {
        let _ = self.cursor.write_i64::<LittleEndian>(value);
    }

    pub fn write_f32(&mut self, value: f32) {
        let _ = self.cursor.write_f32::<LittleEndian>(value);
    }

    pub fn write_f64(&mut self, value: f64) {
        let _ = self.cursor.write_f64::<LittleEndian>(value);
    }
}
