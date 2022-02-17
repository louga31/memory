#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

pub use core::ffi::c_void;

pub type c_char = i8;
pub type c_double = f64;
pub type c_float = f32;
pub type c_int = i32;
pub type c_long = i64;
pub type c_longlong = i64;
pub type c_schar = i8;
pub type c_short = i16;
pub type c_uchar = u8;

pub type c_uint = u32;
pub type c_ulong = u64;
pub type c_ulonglong = u64;
pub type c_ushort = u16;

pub struct DummyProtocolData {
    pub blank: u8,
}

/* Defines used to check if call is really coming from client */
pub static BASE_OPERATION: u32 = 0x7cd4;
pub static VARIABLE_NAME: &'static str = "zLjiCTzRj\0";

/* This is only to modify every command/magic key with only 1 def and don't need to go everywhere, the compiler will automatically parse the operation to number */
pub static COMMAND_MAGIC: u32 = BASE_OPERATION * 0xbb50;

/* Operations */
pub static COPY_OPERATION: u32 = BASE_OPERATION * 0xdf5;
pub static SETUP_OPERATION: u32 = BASE_OPERATION * 0x68c;
pub static GET_PROCESS_BASE_ADDRESS_OPERATION: u32 = BASE_OPERATION * 0x86e;

/* Operation modifiers */
pub static DIRECT_COPY: u32 = 4;

/* Struct containing data used to communicate with the client */
#[repr(C)]
pub struct MemoryCommand {
    pub magic: u32,
    pub operation: u32,
    pub data: [u64; 6],
}

impl Default for MemoryCommand {
    fn default() -> Self {
        MemoryCommand {
            magic: COMMAND_MAGIC,
            operation: 0,
            data: [0; 6],
        }
    }
}