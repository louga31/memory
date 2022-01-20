#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(abi_efiapi)]

mod driver;
#[macro_use]
mod logger;
mod utils;

use core::mem::MaybeUninit;
use core::{mem, ptr::NonNull};

use crate::{driver::set_variable_hook, utils::*};
use protocol::DummyProtocolData;
use uefi::{
    prelude::*,
    proto::console::text::Color,
    table::boot::{EventType, Tpl},
    Event,
};
use utils::boot_services;

unsafe extern "efiapi" fn handle_exit_boot_services(
    _event: Event,
    _context: Option<NonNull<core::ffi::c_void>>,
) {
    info!("[~] ExitBootServices() has been called.");

    let stdout = system_table().stdout();

    let mut status = stdout
        .set_color(Color::White, Color::Green)
        .unwrap()
        .status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to set the color: {:#x}", code);
    }

    status = stdout.clear().unwrap().status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to clear the screen: {:#x}", code);
    }
    println!("Driver seems to be working as expected! Windows is booting now...");
}

unsafe extern "efiapi" fn handle_set_virtual_address_map(
    _event: Event,
    _context: Option<NonNull<core::ffi::c_void>>,
) {
    info!("[~] SetVirtualAddressMap() has been called.");
}

#[entry]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    #[cfg(debug_assertions)]
    {
        // utils::wait_for_debugger();
    }

    unsafe { utils::SYSTEM_TABLE = MaybeUninit::new(system_table.unsafe_clone()) };

    let stdout = system_table.stdout();

    let mut status = stdout
        .set_color(Color::Yellow, Color::Black)
        .unwrap()
        .status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to set the color: {:#x}", code);
    }

    status = stdout.clear().unwrap().status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to clear the screen: {:#x}", code);
    }

    print!("\n\n");
    println!("██╗      ██████╗ ██╗   ██╗ ██████╗  █████╗ ██████╗  ██╗");
    println!("██║     ██╔═══██╗██║   ██║██╔════╝ ██╔══██╗╚════██╗███║");
    println!("██║     ██║   ██║██║   ██║██║  ███╗███████║ █████╔╝╚██║");
    println!("██║     ██║   ██║██║   ██║██║   ██║██╔══██║ ╚═══██╗ ██║");
    println!("███████╗╚██████╔╝╚██████╔╝╚██████╔╝██║  ██║██████╔╝ ██║");
    println!("╚══════╝ ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═════╝  ╚═╝");

    let mut status = stdout
        .set_color(Color::White, Color::Black)
        .unwrap()
        .status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to set the color: {:#x}", code);
    }

    unsafe {
        let status = (raw_boot_services().install_protocol_interface)(
            &handle as *const _ as *mut *mut core::ffi::c_void,
            core::mem::transmute::<&u128, &r_efi::base::Guid>(
                &164914455405085096250244763393526834297,
            ) as *const _ as *mut r_efi::base::Guid,
            r_efi::system::NATIVE_INTERFACE,
            &DummyProtocolData { blank: 0 } as *const _ as *mut core::ffi::c_void,
        );
        if status.is_error() {
            error!(
                "[-] Installing protocol interface failed: {:#x}",
                status.as_usize()
            );
            return Status(status.as_usize());
        }
    }

    // Register to events relevant for runtime drivers.
    unsafe {
        status = boot_services()
            .create_event(
                EventType::SIGNAL_VIRTUAL_ADDRESS_CHANGE,
                Tpl::NOTIFY,
                Some(handle_set_virtual_address_map),
                Some(NonNull::new(
                    runtime_services() as *const _ as *mut core::ffi::c_void
                ))
                .unwrap(),
            )
            .unwrap()
            .status();
    };

    if status.is_error() {
        let Status(code) = status;
        error!(
            "[-] Creating VIRTUAL_ADDRESS_CHANGE event failed: {:#x}",
            code
        );
        return status;
    }

    unsafe {
        status = boot_services()
            .create_event(
                EventType::SIGNAL_EXIT_BOOT_SERVICES,
                Tpl::NOTIFY,
                Some(handle_exit_boot_services),
                Some(NonNull::new(
                    runtime_services() as *const _ as *mut core::ffi::c_void
                ))
                .unwrap(),
            )
            .unwrap()
            .status();
    }

    if status.is_error() {
        let Status(code) = status;
        error!("[-] Creating EXIT_BOOT_SERVICES event failed: {:#x}", code);
        return status;
    }

    info!("Hooking set_variable()...");
    let set_variable = raw_runtime_services().set_variable;
    info!("[+] set_variable = {:x}", set_variable as usize);

    let region = unwrap!(region_containing(set_variable as _));
    info!("[+] region = {:x}:{:x}", region.start, region.end);
    let region = unsafe { range_to_slice(region) };

    let location = unwrap!(search_for_contiguous(region, 0, unsafe {
        set_variable_hook::len()
    }));
    let start = location.as_ptr() as usize;
    info!("[+] location = {:x}:{:x}", start, start + location.len());

    unsafe {
        let hook = set_variable_hook::copy_to(location);
        hook.hook(mem::transmute(set_variable));

        let guard = system_table.boot_services().raise_tpl(Tpl::NOTIFY);
        hook.toggle();
        mem::drop(guard);
    }

    status = system_table
        .stdout()
        .set_color(Color::Green, Color::Black)
        .unwrap()
        .status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to set the color: {:#x}", code);
    }

    println!("Driver has been loaded successfully. You can now boot to the OS.");
    println!("If you don't see a green screen while booting disable Secure Boot!.");

    status = system_table
        .stdout()
        .set_color(Color::White, Color::Black)
        .unwrap()
        .status();
    if status.is_error() {
        let Status(code) = status;
        error!("[-] Failed to set the color: {:#x}", code);
    }
    // Your runtime driver initialization. If the initialization fails, manually close the previously
    // created events with:
    // (boot_services().close_event)(event_virtual_address);
    // (boot_services().close_event)(event_boot_services);

    info!("[~] EFI runtime driver has been loaded and initialized.");

    Status::SUCCESS
}
