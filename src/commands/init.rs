use crate::core::constants::NARU_DIR;
use crate::core::persistence;
use anyhow::Result;
use std::path::Path;

pub struct InitCommand;

impl InitCommand {
    pub fn new() -> Self {
        InitCommand
    }

    pub fn execute(&self) -> Result<()> {
        if Path::new(NARU_DIR).exists() {
            println!("Project already initialized.");
        } else {
            persistence::init_project()?;
            println!("Project initialized successfully.");
        }
        Ok(())
    }
}

impl Default for InitCommand {
    fn default() -> Self {
        Self::new()
    }
}
