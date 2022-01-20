#![allow(dead_code)]

use core::{arch::asm, mem::MaybeUninit, ops::Range, slice};

use r_efi::{
    system::BootServices as RawBootServices, system::RuntimeServices as RawRuntimeServices,
};
use uefi::{prelude::*, Completion};

use crate::error;

pub static mut SYSTEM_TABLE: MaybeUninit<SystemTable<Boot>> = MaybeUninit::uninit();

pub fn system_table() -> &'static mut SystemTable<Boot> {
    unsafe { &mut *SYSTEM_TABLE.as_mut_ptr() }
}

pub fn raw_runtime_services() -> &'static RawRuntimeServices {
    unsafe { &*(system_table().runtime_services() as *const _ as *const _) }
}

pub fn runtime_services() -> &'static RuntimeServices {
    &*system_table().runtime_services()
}

pub fn raw_boot_services() -> &'static RawBootServices {
    unsafe { &*(system_table().boot_services() as *const _ as *const _) }
}

pub fn boot_services() -> &'static BootServices {
    &*system_table().boot_services()
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($crate::utils::system_table().stdout(), $($arg)*);
    }}
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($crate::utils::system_table().stdout(), $($arg)*);
    }}
}

#[macro_export]
macro_rules! unwrap {
    ($expr:expr) => {
        $expr?.split().1
    };
}

static mut GDB_ATTACHED: bool = false;

pub fn wait_for_debugger() {
    unsafe {
        while !GDB_ATTACHED {
            asm!("pause");
        }
    }
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "[-] Panic in {} at ({}, {}):",
            location.file(),
            location.line(),
            location.column()
        );
        if let Some(message) = info.message() {
            error!("[-] {}", message);
        }
    }

    loop {}
}

static mut BUFFER: [u8; 4096] = [0; 4096];

pub fn region_containing(address: usize) -> uefi::Result<Range<usize>> {
    let (status, (_, descriptors)) = system_table()
        .boot_services()
        .memory_map(unsafe { &mut BUFFER })?
        .split();

    let region = descriptors
        .map(|descriptor| {
            let start = descriptor.phys_start as usize;
            let end = start + descriptor.page_count as usize * 4096;

            start..end
        })
        .find(|region| region.contains(&address));

    match region {
        Some(region) => Ok(Completion::new(status, region)),
        None => Err(Status::NOT_FOUND.into()),
    }
}

pub unsafe fn range_to_slice(range: Range<usize>) -> &'static mut [u8] {
    slice::from_raw_parts_mut(range.start as _, range.len())
}

pub fn search_for_contiguous(slice: &mut [u8], item: u8, count: usize) -> uefi::Result<&mut [u8]> {
    let mut current = 0;

    for (n, i) in slice.iter().enumerate() {
        if *i == item {
            current += 1;

            if current == count {
                let slice = &mut slice[n + 1 - count..n + 1];

                return Ok(slice.into());
            }
        } else if current != 0 {
            current = 0;
        }
    }

    Err(Status::NOT_FOUND.into())
}
