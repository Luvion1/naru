use crate::core::constants::NARU_DIR;
use anyhow::Result;

pub struct AuditLogCommand {
    pub count: usize,
}

pub fn execute_log(cmd: &AuditLogCommand) -> Result<()> {
    let log_path = format!("{NARU_DIR}/audit.log");
    let logs = crate::core::audit::AuditLogEntry::get_recent_logs(&log_path, cmd.count)
        .map_err(|e| anyhow::anyhow!("Failed to read audit logs: {e}"))?;

    if logs.is_empty() {
        println!("No audit logs found.");
    } else {
        println!("\nRecent Audit Logs (last {} entries):", cmd.count);
        println!("{}", "-".repeat(100));
        println!(
            "{:<20} | {:<10} | {:<12} | {:<15} | {:<15} | {:<10}",
            "Timestamp", "Action", "Env", "Key", "User", "Hash"
        );
        println!("{}", "-".repeat(100));

        for log in logs {
            let key_str = log.key.as_deref().unwrap_or("-");
            let user_str = log.user.as_deref().unwrap_or("unknown");
            let hash_short = log.hash.as_ref().map_or("none", |h| &h[..8]);

            println!(
                "{:<20} | {:<10} | {:<12} | {:<15} | {:<15} | {:<10}",
                log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                log.action,
                log.environment,
                key_str,
                user_str,
                hash_short
            );
        }
        println!("{}", "-".repeat(100));
    }
    Ok(())
}

pub fn execute_verify() {
    let log_path = format!("{NARU_DIR}/audit.log");
    println!("Verifying audit log integrity...");
    match crate::core::audit::AuditLogEntry::verify_log_integrity(&log_path) {
        Ok(true) => {
            println!("✅ Audit log integrity verified. No tampering detected.");
        }
        Ok(false) => {
            println!("❌ CRITICAL: Audit log integrity check FAILED!");
            println!("The audit trail may have been tampered with or corrupted.");
        }
        Err(e) => {
            println!("⚠️ Error during verification: {e}");
        }
    }
}
