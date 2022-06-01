use std::process::Command;

use serde::Deserialize;

use crate::vk::{KeyInputManager, VKSeq};
use crate::Error;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum MappingAction {
    Keys { keys: VKSeq },
    Program { path: String },
}

pub struct Mapping(pub u8, pub MappingAction);

impl Mapping {
    pub fn down(&self, key_manager: &KeyInputManager) -> Result<(), Error> {
        match &self.1 {
            MappingAction::Keys { keys } => {
                key_manager.down(self.0, keys).map_err(Error::Windows)?;
            }
            MappingAction::Program { path } => {
                Command::new(path).spawn().map_err(Error::Io)?;
            }
        }
        Ok(())
    }

    pub fn up(&self, key_manager: &KeyInputManager) -> Result<(), Error> {
        match &self.1 {
            MappingAction::Keys { keys } => {
                key_manager.up(self.0, keys).map_err(Error::Windows)?;
            }
            MappingAction::Program { .. } => {
                //
            }
        }
        Ok(())
    }
}
