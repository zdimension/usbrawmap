use std::ffi::c_void;
use std::io::Read;
use std::path::Path;
use std::{io, ptr};
use widestring::WideCString;

use windows::core::{Error as WinError, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, BOOLEAN, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    FILE_SHARE_NONE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;

use crate::{IoctlMessage, SetupBuffer, StartFiltering, DEFAULT_INTERNAL_KERNEL_BUFFER_SIZE};

pub struct UsbPcapDriver {
    handle: HANDLE,
}

impl Drop for UsbPcapDriver {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

impl UsbPcapDriver {
    pub fn new(filename: &Path) -> Result<UsbPcapDriver, WinError> {
        let filename = unsafe { WideCString::from_os_str_unchecked(filename.as_os_str()) };
        let handle = unsafe {
            CreateFileW(
                PCWSTR(filename.as_ptr() as *const _),
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                FILE_SHARE_NONE,
                ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE::default(),
            )
        }?;

        let res = UsbPcapDriver { handle };

        res.reset_buffer()?;
        res.ioctl(StartFiltering {
            addresses: [0, 0, 0, 0],
            filter_all: BOOLEAN(1),
        })?;

        Ok(res)
    }

    pub(crate) fn reset_buffer(&self) -> Result<(), WinError> {
        self.ioctl(SetupBuffer {
            size: DEFAULT_INTERNAL_KERNEL_BUFFER_SIZE as u32,
        })
    }

    fn ioctl<T: IoctlMessage>(&self, ioctl_message: T) -> Result<(), WinError> {
        let buf = &ioctl_message as *const _ as *mut c_void;
        unsafe {
            DeviceIoControl(
                self.handle,
                T::ID,
                buf,
                std::mem::size_of::<T>() as u32,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
        .ok()
    }

    unsafe fn read_internal(&self, buf: &mut [u8]) -> Result<usize, WinError> {
        let mut read = 0;
        ReadFile(
            self.handle,
            buf.as_mut_ptr() as *mut c_void,
            buf.len() as u32,
            &mut read,
            ptr::null_mut(),
        )
        .ok()?;
        if read == 0 {
            self.reset_buffer()?;
            self.read_internal(buf)
        } else {
            Ok(read as usize)
        }
    }
}

impl Read for UsbPcapDriver {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        unsafe { self.read_internal(buf) }.map_err(|e| io::Error::from_raw_os_error(e.code().0))
    }
}
