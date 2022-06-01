use windows::Win32::Foundation::BOOLEAN;
use windows::Win32::System::Ioctl::{
    FILE_DEVICE_UNKNOWN, FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED,
};

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

pub trait IoctlMessage {
    const ID: u32;
}

#[repr(C)]
pub struct SetupBuffer {
    pub size: u32,
}

impl IoctlMessage for SetupBuffer {
    const ID: u32 = ctl_code(
        FILE_DEVICE_UNKNOWN,
        0x800,
        METHOD_BUFFERED,
        FILE_READ_ACCESS,
    );
}

#[repr(C, packed)]
pub struct StartFiltering {
    pub addresses: [u32; 4],
    pub filter_all: BOOLEAN,
}

impl IoctlMessage for StartFiltering {
    const ID: u32 = ctl_code(
        FILE_DEVICE_UNKNOWN,
        0x801,
        METHOD_BUFFERED,
        FILE_READ_ACCESS | FILE_WRITE_ACCESS,
    );
}
