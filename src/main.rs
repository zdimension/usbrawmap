use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::thread;

use anyhow::{bail, Context};
use bitflags::bitflags;
use config_file::FromConfigFile;
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::{LegacyPcapReader, PcapBlockOwned, PcapError};
use pnet_packet::usbpcap::{UsbPcapFunction, UsbPcapPacket};
use pnet_packet::Packet;
use serde::{Deserialize, Deserializer};
use thiserror::Error;
use windows::core::Error as WinError;

use crate::driver::UsbPcapDriver;
use crate::ioctl::{IoctlMessage, SetupBuffer, StartFiltering};
use crate::mappings::{Mapping, MappingAction};
use crate::vk::KeyInputManager;

mod driver;
mod ioctl;
mod mappings;
mod vk;

const DEFAULT_INTERNAL_KERNEL_BUFFER_SIZE: usize = 4096;

const DLT_USBPCAP: i32 = 249;
const URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER: UsbPcapFunction = UsbPcapFunction(0x0009);
const URB_INTERRUPT: u8 = 0x01;
const DIRECTION_IN: u8 = 1;

bitflags! {
    struct Modifiers: u8 {
        const L_CTRL = 0b0000_0001;
        const L_SHIFT = 0b0000_0010;
        const L_ALT = 0b0000_0100;
        const L_WIN = 0b0000_1000;
        const R_CTRL = 0b0001_0000;
        const R_SHIFT = 0b0010_0000;
        const R_ALT = 0b0100_0000;
        const R_WIN = 0b1000_0000;
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct KeyboardReport {
    modifiers: Modifiers,
    reserved: u8,
    keys: [u8; 6],
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error: {0}")]
    Message(&'static str),
    #[error("Win32 error: {0}")]
    Windows(WinError),
    #[error("Pcap error: {0}")]
    Pcap(PcapError<&'static [u8]>),
    #[error("I/O error: {0}")]
    Io(std::io::Error),
    #[error("Config error: {0}")]
    Config(config_file::ConfigFileError),
}

#[derive(Deserialize)]
pub struct Config {
    general: GeneralConfig,
    mappings: MappingConfig,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    driver: u8,
}

pub struct MappingConfig(HashMap<u8, Mapping>);

impl<'de> Deserialize<'de> for MappingConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        HashMap::<String, MappingAction>::deserialize(deserializer)?
            .into_iter()
            .map(|(k, v)| {
                k.parse()
                    .map(|k| (k, (Mapping(k, v))))
                    .map_err(|e| serde::de::Error::custom(e))
            })
            .collect::<Result<_, D::Error>>()
            .map(MappingConfig)
    }
}

fn main() -> anyhow::Result<()> {
    let config = Config::from_config_file("usbrawmap.toml").context("Failed to load config")?;

    println!("Loaded {} mappings", config.mappings.0.len());

    let thread = thread::spawn(move || -> Result<(), anyhow::Error> {
        let driver = UsbPcapDriver::new(&Path::new(&format!(
            r"\\.\USBPcap{}",
            config.general.driver
        )))
        .context("Failed to open driver")?;
        let mut reader = LegacyPcapReader::new(DEFAULT_INTERNAL_KERNEL_BUFFER_SIZE, driver)
            .context("Failed to create reader")?;
        let mut old_keys = HashSet::new();
        let km = KeyInputManager::new();
        loop {
            match reader.next() {
                Ok((offset, block)) => {
                    match block {
                        PcapBlockOwned::LegacyHeader(hdr) => {
                            if hdr.network.0 == DLT_USBPCAP {
                                println!("USBPcap header found");
                            } else {
                                bail!(Error::Message("Not USBPcap"));
                            }
                        }
                        PcapBlockOwned::Legacy(b) => {
                            let packet = UsbPcapPacket::new(b.data)
                                .context("Failed to decode USBPcap packet")?;
                            if packet.get_transfer() == URB_INTERRUPT
                                && packet.get_endpoint() == 1
                                && packet.get_direction() == DIRECTION_IN
                                && packet.get_data_length() == 8
                                && packet.get_function() == URB_FUNCTION_BULK_OR_INTERRUPT_TRANSFER
                            {
                                let report: &KeyboardReport = unsafe {
                                    &*(packet.payload().as_ptr() as *const KeyboardReport)
                                };
                                let new_keys = HashSet::from(report.keys);
                                let pressed = &new_keys - &old_keys;
                                let released = &old_keys - &new_keys;
                                for key in pressed {
                                    if let Some(mapping) = config.mappings.0.get(&key) {
                                        mapping.down(&km)?;
                                    }
                                }
                                for key in released {
                                    if let Some(mapping) = config.mappings.0.get(&key) {
                                        mapping.up(&km)?;
                                    }
                                }
                                old_keys = new_keys;
                            }
                        }
                        PcapBlockOwned::NG(_) => unreachable!(),
                    }
                    reader.consume(offset);
                }
                Err(PcapError::Eof | PcapError::Incomplete) => {
                    reader.refill().unwrap();
                }
                Err(e) => {
                    bail!(Error::Pcap(e.to_owned_vec()));
                }
            }
        }
    });

    match thread.join() {
        Ok(Ok(_)) => unreachable!(),
        Ok(Err(err)) => Err(err),
        Err(_) => bail!("Thread panicked"),
    }
}
