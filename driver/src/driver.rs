#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::mem::{self, MaybeUninit};

use ezhook::remote_swap_hook;
use uefi::{prelude::*, Guid};

static VARIABLE_NAME: &str = "zLjiCTzRj\0";

// use crate::protocol::*;
use protocol::{
    MemoryCommand, COMMAND_MAGIC, COPY_OPERATION, DIRECT_COPY, GET_PROCESS_BASE_ADDRESS_OPERATION,
    SETUP_OPERATION,
};

// use crate::protocol::VARIABLE_NAME;

pub use core::ffi::c_void;

#[allow(dead_code)]
pub type c_char = i8;
#[allow(dead_code)]
pub type c_double = f64;
#[allow(dead_code)]
pub type c_float = f32;
#[allow(dead_code)]
pub type c_int = i32;
#[allow(dead_code)]
pub type c_long = i64;
#[allow(dead_code)]
pub type c_longlong = i64;
#[allow(dead_code)]
pub type c_schar = i8;
#[allow(dead_code)]
pub type c_short = i16;
#[allow(dead_code)]
pub type c_uchar = u8;
#[allow(dead_code)]

pub type c_uint = u32;
#[allow(dead_code)]
pub type c_ulong = u64;
#[allow(dead_code)]
pub type c_ulonglong = u64;
#[allow(dead_code)]
pub type c_ushort = u16;

macro_rules! cast_to_function {
    ($address:expr, $t:ty) => {
        core::mem::transmute::<*const (), $t>($address as _)
    };
}

static mut GetProcessByPid: MaybeUninit<PsLookupProcessByProcessId> = MaybeUninit::uninit();
static mut GetBaseAddress: MaybeUninit<PsGetProcessSectionBaseAddress> = MaybeUninit::uninit();
static mut MCopyVirtualMemory: MaybeUninit<MmCopyVirtualMemory> = MaybeUninit::uninit();

type PsLookupProcessByProcessId =
    unsafe extern "C" fn(ProcessId: *mut c_void, OutPEProcess: *mut *mut c_void) -> c_int;
type PsGetProcessSectionBaseAddress = unsafe extern "C" fn(PEProcess: *mut c_void) -> *mut c_void;
type MmCopyVirtualMemory = unsafe extern "C" fn(
    SourceProcess: *mut c_void,
    SourceAddress: *mut c_void,
    TargetProcess: *mut c_void,
    TargetAddress: *mut c_void,
    BufferSize: usize,
    PreviousMode: c_char,
    ReturnSize: *mut c_longlong,
) -> c_int;

remote_swap_hook! {
    #[hook]
    pub unsafe extern "efiapi" fn set_variable_hook(
        variable_name: *const u16,
        vendor_guid: *const Guid,
        attributes: u32,
        data_size: usize,
        data: *const u8,
    ) -> Status {
        if !variable_name.is_null() {
            if eq(variable_name, VARIABLE_NAME.as_bytes()) {
                if data_size == mem::size_of::<MemoryCommand>() {
                    /* We did it! */
                    /* Now we can call the magic function */
                    return run_command(&*(data as *const MemoryCommand));
                }
                return Status::SUCCESS
            }
        }
        orig!(variable_name, vendor_guid, attributes, data_size, data)
    }

    unsafe fn eq(a: *const u16, b: &[u8]) -> bool {
        b.iter().enumerate().all(|(n, i)| *a.add(n) == *i as u16)
    }

    #[repr(C)]
    struct CopyData {
        src: *const u8,
        dst: *mut u8,
        count: usize,
    }
    unsafe fn copy(data: &CopyData) {
        for i in 0..data.count {
            *data.dst.add(i) = *data.src.add(i);
        }
    }

    unsafe fn run_command(command: &MemoryCommand) -> Status{
        if command.magic == COMMAND_MAGIC {
            if command.operation == COPY_OPERATION {
                let data = command.data;

                let src_process_id = data[0] as usize;
                let src_address = data[1] as *mut c_void;
                let dest_process_id = data[2] as usize;
                let dest_address = data[3] as *mut c_void;
                let size = data[4] as usize;
                let result_address = data[5] as *mut i32;

                if src_process_id == DIRECT_COPY as usize {
                    copy(&CopyData {
                        src: src_address as *const u8,
                        dst: dest_address as *mut u8,
                        count: size,
                    });
                } else {
                    let mut src_process: *mut c_void = 0 as *mut c_void;
                    let mut dest_process: *mut c_void = 0 as *mut c_void;
                    let mut size_out: i64 = 0;
                    let mut status;

                    status = GetProcessByPid.assume_init()(src_process_id as *mut c_void, &mut src_process);
                    if status < 0 {
                        *result_address = status;
                        return Status::SUCCESS;
                    }

                    status = GetProcessByPid.assume_init()(dest_process_id as *mut c_void, &mut dest_process);
                    if status < 0 {
                        *result_address = status;
                        return Status::SUCCESS;
                    }

                    *result_address = MCopyVirtualMemory.assume_init()(src_process, src_address, dest_process, dest_address, size, 1 as c_char, &mut size_out);
                }
                return Status::SUCCESS;
            }
            else if command.operation == SETUP_OPERATION {
                let data = command.data;

                GetProcessByPid = MaybeUninit::new(cast_to_function!(data[0], PsLookupProcessByProcessId));
                GetBaseAddress = MaybeUninit::new(cast_to_function!(data[1], PsGetProcessSectionBaseAddress));
                MCopyVirtualMemory = MaybeUninit::new(cast_to_function!(data[2], MmCopyVirtualMemory));

                let result_address = data[3] as *mut i32;
                *result_address = 1;

                return Status::SUCCESS;
            } else if command.operation == GET_PROCESS_BASE_ADDRESS_OPERATION {
                let data = command.data;

                let pid: *mut c_void = data[0] as *mut c_void;
                let result_address = data[1] as *mut u64;
                let mut process_ptr = 0 as *mut c_void;

                //Find process by ID
                if GetProcessByPid.assume_init()(pid, &mut process_ptr) < 0 || process_ptr == 0 as *mut c_void {
                    *result_address = 0; // Process not found
                    return Status::SUCCESS;
                }

                //Find process Base Address
                *result_address = GetBaseAddress.assume_init()(process_ptr) as u64; //Return Base Address

                return Status::SUCCESS;
            }
            return Status::UNSUPPORTED;
        }
        return Status::INVALID_PARAMETER;
    }
}
