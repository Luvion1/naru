mod cli;
mod commands;
mod core;
#[cfg(test)]
mod deep_security_tests;
#[cfg(test)]
mod penetration_tests;
#[cfg(test)]
mod security_tests;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::parser::Cli::parse();

    match cli.command {
        cli::parser::Commands::Version => {
            println!("Naru version {}", env!("CARGO_PKG_VERSION"));
        }
        cli::parser::Commands::Init => {
            commands::init::InitCommand::new().execute()?;
        }
        cli::parser::Commands::Set { kv, env, secret } => {
            let (key, value) = kv
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid format. Use key=value"))?;
            commands::set::SetCommand::new(key.to_string(), value.to_string(), env, secret)
                .execute()?;
        }
        cli::parser::Commands::Get { key, env } => {
            commands::get::GetCommand::new(key, env).execute()?;
        }
        cli::parser::Commands::List { env } => {
            commands::list::ListCommand::new(env).execute()?;
        }
        cli::parser::Commands::Import { file_path, env } => {
            commands::import::ImportCommand::new(file_path, env).execute()?;
        }
        cli::parser::Commands::Export {
            file_path,
            env,
            format,
        } => {
            commands::export::ExportCommand::new(file_path, env, format).execute()?;
        }
        cli::parser::Commands::Schema { action } => match action {
            cli::parser::SchemaAction::Add {
                key,
                r#type,
                description,
                secret,
            } => {
                commands::schema::execute_add(commands::schema::SchemaAddCommand {
                    key,
                    r#type,
                    description,
                    secret,
                })?;
            }
            cli::parser::SchemaAction::Remove { key } => {
                commands::schema::execute_remove(commands::schema::SchemaRemoveCommand { key })?;
            }
            cli::parser::SchemaAction::View => {
                commands::schema::execute_view()?;
            }
            cli::parser::SchemaAction::Edit { key } => {
                commands::schema::execute_edit(commands::schema::SchemaEditCommand { key })?;
            }
        },
        cli::parser::Commands::Env { action } => match action {
            cli::parser::EnvAction::Add { name } => {
                commands::env::execute_add(commands::env::EnvAddCommand { name })?;
            }
            cli::parser::EnvAction::Remove { name } => {
                commands::env::execute_remove(commands::env::EnvRemoveCommand { name })?;
            }
            cli::parser::EnvAction::SetParent { name, parent } => {
                commands::env::execute_set_parent(commands::env::EnvSetParentCommand {
                    name,
                    parent,
                })?;
            }
            cli::parser::EnvAction::List => {
                commands::env::execute_list()?;
            }
        },
        cli::parser::Commands::Backup { action } => match action {
            cli::parser::BackupAction::Create { file_path } => {
                commands::backup::execute_create(commands::backup::BackupCreateCommand {
                    file_path,
                })?;
            }
            cli::parser::BackupAction::Restore { file_path } => {
                commands::backup::execute_restore(commands::backup::BackupRestoreCommand {
                    file_path,
                })?;
            }
        },
        cli::parser::Commands::Diff { env1, env2 } => {
            commands::diff::DiffCommand::new(env1, env2).execute()?;
        }
        cli::parser::Commands::Convert {
            from_file,
            to_file,
            from_format,
            to_format,
        } => {
            commands::convert::ConvertCommand::new(from_file, to_file, from_format, to_format)
                .execute()?;
        }
        cli::parser::Commands::Crypto { action } => match action {
            cli::parser::CryptoAction::Encrypt {
                input_file,
                output_file,
            } => {
                commands::crypto::execute_encrypt(commands::crypto::CryptoEncryptCommand {
                    input_file,
                    output_file,
                })?;
            }
            cli::parser::CryptoAction::Decrypt {
                input_file,
                output_file,
            } => {
                commands::crypto::execute_decrypt(commands::crypto::CryptoDecryptCommand {
                    input_file,
                    output_file,
                })?;
            }
        },
        cli::parser::Commands::Audit { action } => match action {
            cli::parser::AuditAction::Log { count } => {
                commands::audit::execute_log(&commands::audit::AuditLogCommand { count })?;
            }
            cli::parser::AuditAction::Verify => {
                commands::audit::execute_verify();
            }
        },
        cli::parser::Commands::Validate => {
            commands::validate::ValidateCommand::new().execute()?;
        }
        cli::parser::Commands::Batch { action } => match action {
            cli::parser::BatchAction::Set { file, env } => {
                commands::batch::batch_set(&file, &env)?;
            }
            cli::parser::BatchAction::Get { keys, env } => {
                commands::batch::batch_get(&keys, &env)?;
            }
            cli::parser::BatchAction::All { env } => {
                commands::batch::batch_get_all(&env)?;
            }
        },
        cli::parser::Commands::Template { action } => match action {
            cli::parser::TemplateAction::Create {
                name,
                include_secrets,
            } => {
                commands::template::template_create(&name, include_secrets)?;
            }
            cli::parser::TemplateAction::Apply { name, env } => {
                commands::template::template_apply(&name, &env)?;
            }
            cli::parser::TemplateAction::List => {
                commands::template::template_list()?;
            }
        },
        cli::parser::Commands::Search { query, env, values } => {
            commands::search::SearchCommand::new(query, env, values).execute()?;
        }
        cli::parser::Commands::Doctor => {
            commands::doctor::DoctorCommand::new().execute()?;
        }
        cli::parser::Commands::Stats => {
            commands::stats::StatsCommand::new().execute()?;
        }
    }

    Ok(())
}
