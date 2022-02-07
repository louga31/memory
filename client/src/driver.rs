use std::mem::size_of;

use crate::protocol::*;
use ntapi::{ntexapi::NtSetSystemEnvironmentValueEx, ntrtl::RtlAdjustPrivilege};
use winapi::shared::{
    guiddef::GUID,
    ntdef::{BOOLEAN, NTSTATUS, NT_SUCCESS, PWCH, ULONG, UNICODE_STRING, USHORT},
};

#[allow(dead_code)]
static EFI_VARIABLE_NON_VOLATILE: ULONG = 0x00000001;
#[allow(dead_code)]
static EFI_VARIABLE_BOOTSERVICE_ACCESS: ULONG = 0x00000002;
#[allow(dead_code)]
static EFI_VARIABLE_RUNTIME_ACCESS: ULONG = 0x00000004;
#[allow(dead_code)]
static EFI_VARIABLE_HARDWARE_ERROR_RECORD: ULONG = 0x00000008;
#[allow(dead_code)]
static EFI_VARIABLE_AUTHENTICATED_WRITE_ACCESS: ULONG = 0x00000010;
#[allow(dead_code)]
static EFI_VARIABLE_TIME_BASED_AUTHENTICATED_WRITE_ACCESS: ULONG = 0x00000020;
#[allow(dead_code)]
static EFI_VARIABLE_APPEND_WRITE: ULONG = 0x00000040;
#[allow(dead_code)]
static ATTRIBUTES: ULONG =
    EFI_VARIABLE_NON_VOLATILE | EFI_VARIABLE_BOOTSERVICE_ACCESS | EFI_VARIABLE_RUNTIME_ACCESS;
#[allow(dead_code)]
static SE_SYSTEM_ENVIRONMENT_PRIVILEGE: ULONG = 22;

pub fn set_system_environment_privilege(enable: bool, was_enabled: &mut bool) -> NTSTATUS {
    *was_enabled = false;
    let mut se_system_environment_was_enabled: BOOLEAN = false as _;
    let status: NTSTATUS;
    unsafe {
        status = RtlAdjustPrivilege(
            SE_SYSTEM_ENVIRONMENT_PRIVILEGE,
            enable as _,
            false as _,
            &mut se_system_environment_was_enabled,
        );
    }
    if NT_SUCCESS(status) {
        *was_enabled = se_system_environment_was_enabled != 0;
    }

    status
}

pub fn send_command(cmd: &mut MemoryCommand) {
    let mut variable_name: UNICODE_STRING = UNICODE_STRING {
        Length: VARIABLE_NAME.chars().count() as USHORT,
        MaximumLength: VARIABLE_NAME.len() as USHORT,
        Buffer: VARIABLE_NAME as *const _ as PWCH,
    };
    let mut dummy_guid = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    unsafe {
        NtSetSystemEnvironmentValueEx(
            &mut variable_name,
            &mut dummy_guid,
            cmd as *mut _ as *mut _,
            size_of::<MemoryCommand>().try_into().unwrap(),
            ATTRIBUTES,
        );
    }
}

pub fn get_base_address(pid: u64) -> u64 {
    let mut result: u64 = 0;
    let mut cmd: MemoryCommand = MemoryCommand::default();
    cmd.operation = GET_PROCESS_BASE_ADDRESS_OPERATION;
    cmd.data[0] = pid;
    cmd.data[1] = &mut result as *const _ as u64;
    send_command(&mut cmd);
    result
}
