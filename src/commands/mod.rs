pub mod audit;
pub mod backup;
pub mod batch;
pub mod convert;
pub mod crypto;
pub mod diff;
pub mod doctor;
pub mod env;
pub mod export;
pub mod get;
pub mod import;
pub mod init;
pub mod list;
pub mod schema;
pub mod search;
pub mod set;
pub mod stats;
pub mod template;
pub mod validate;

use anyhow::Result;

#[allow(dead_code)]
pub trait Command {
    fn execute(&self) -> Result<()>;
}
