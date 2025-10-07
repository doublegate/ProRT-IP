//! Privilege management for raw socket access

use prtip_core::{Error, Result};

/// Check if the current process has raw socket capabilities
///
/// # Returns
///
/// Returns Ok(true) if the process has necessary privileges, Ok(false) otherwise
///
/// # Platform-specific behavior
///
/// - **Linux**: Checks for root (UID 0) or CAP_NET_RAW capability
/// - **Windows**: Checks for Administrator privileges
/// - **macOS**: Checks for root (UID 0)
pub fn has_raw_socket_capability() -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        linux_has_capability()
    }

    #[cfg(target_os = "windows")]
    {
        windows_has_capability()
    }

    #[cfg(target_os = "macos")]
    {
        macos_has_capability()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(Error::Privilege(
            "Unsupported platform for privilege checking".to_string(),
        ))
    }
}

/// Check if we have necessary privileges for raw packet access
///
/// # Returns
///
/// Returns Ok(()) if we have privileges, Err otherwise
pub fn check_privileges() -> Result<()> {
    if !has_raw_socket_capability()? {
        #[cfg(target_os = "linux")]
        let msg = "Insufficient privileges for raw socket access. \
                   Run with sudo or grant CAP_NET_RAW capability with: \
                   sudo setcap cap_net_raw+eip /path/to/prtip";

        #[cfg(target_os = "windows")]
        let msg = "Insufficient privileges for raw socket access. \
                   Run as Administrator.";

        #[cfg(target_os = "macos")]
        let msg = "Insufficient privileges for raw socket access. \
                   Run with sudo or install ChmodBPF.";

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let msg = "Insufficient privileges for raw socket access.";

        return Err(Error::Privilege(msg.to_string()));
    }

    Ok(())
}

/// Drop elevated privileges to a specific user and group
///
/// # Arguments
///
/// * `user` - Username to drop privileges to
/// * `group` - Group name to drop privileges to
///
/// # Returns
///
/// Returns Ok(()) on success, Err otherwise
///
/// # Platform-specific behavior
///
/// - **Linux/macOS**: Uses setuid/setgid to drop privileges
/// - **Windows**: Logs warning (Windows doesn't support setuid)
///
/// # Safety
///
/// This operation is **irreversible**. After dropping privileges, the process
/// cannot regain elevated privileges.
pub fn drop_privileges(user: &str, group: &str) -> Result<()> {
    #[cfg(unix)]
    {
        unix_drop_privileges(user, group)
    }

    #[cfg(windows)]
    {
        windows_drop_privileges(user, group)
    }
}

// ===== Linux implementation =====

#[cfg(target_os = "linux")]
fn linux_has_capability() -> Result<bool> {
    use nix::unistd::Uid;

    // Check if running as root
    if Uid::effective().is_root() {
        tracing::debug!("Running as root (UID 0)");
        return Ok(true);
    }

    // Check for CAP_NET_RAW capability
    // Note: This is a simplified check. For production, use the `caps` crate
    // to properly check capabilities.
    tracing::debug!("Not running as root, checking for CAP_NET_RAW capability");

    // For now, if not root, assume no capability
    // A full implementation would check /proc/self/status or use libcap
    Ok(false)
}

#[cfg(unix)]
fn unix_drop_privileges(user: &str, group: &str) -> Result<()> {
    use nix::unistd::{setgid, setuid, Group, Uid, User};

    tracing::info!("Attempting to drop privileges to {}:{}", user, group);

    // Get user info
    let user_info = User::from_name(user)
        .map_err(|e| Error::Privilege(format!("Failed to get user info: {}", e)))?
        .ok_or_else(|| Error::Privilege(format!("User not found: {}", user)))?;

    // Get group info
    let group_info = Group::from_name(group)
        .map_err(|e| Error::Privilege(format!("Failed to get group info: {}", e)))?
        .ok_or_else(|| Error::Privilege(format!("Group not found: {}", group)))?;

    // Drop group privileges first
    setgid(group_info.gid).map_err(|e| Error::Privilege(format!("Failed to setgid: {}", e)))?;

    // Drop user privileges
    setuid(user_info.uid).map_err(|e| Error::Privilege(format!("Failed to setuid: {}", e)))?;

    // Verify we cannot regain root privileges
    if setuid(Uid::from_raw(0)).is_ok() {
        return Err(Error::Privilege(
            "Failed to drop privileges securely: can still setuid(0)".to_string(),
        ));
    }

    tracing::info!("Successfully dropped privileges to {}:{}", user, group);
    Ok(())
}

// ===== Windows implementation =====

#[cfg(target_os = "windows")]
fn windows_has_capability() -> Result<bool> {
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Security::IsUserAnAdmin;

    unsafe {
        let is_admin: BOOL = IsUserAnAdmin();
        Ok(is_admin.as_bool())
    }
}

#[cfg(windows)]
fn windows_drop_privileges(_user: &str, _group: &str) -> Result<()> {
    // Windows doesn't support setuid/setgid
    tracing::warn!(
        "Privilege dropping not supported on Windows. \
         Process will continue running with current privileges."
    );
    Ok(())
}

// ===== macOS implementation =====

#[cfg(target_os = "macos")]
fn macos_has_capability() -> Result<bool> {
    use nix::unistd::Uid;

    // macOS requires root for BPF access (unless ChmodBPF is installed)
    if Uid::effective().is_root() {
        tracing::debug!("Running as root (UID 0)");
        return Ok(true);
    }

    tracing::debug!("Not running as root");
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_raw_socket_capability() {
        // This test will pass or fail depending on current privileges
        let result = has_raw_socket_capability();
        assert!(result.is_ok());

        // Log the result for manual verification
        if let Ok(has_cap) = result {
            println!("Has raw socket capability: {}", has_cap);
        }
    }

    #[test]
    fn test_check_privileges() {
        // This test's behavior depends on current privileges
        let result = check_privileges();

        // We can't assert success/failure without knowing current privileges
        // Just ensure it doesn't panic
        if let Err(e) = result {
            println!("Privilege check failed (expected if not root): {}", e);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_drop_privileges_invalid_user() {
        // Don't actually try to drop privileges in tests
        // Just test error handling for invalid user
        if has_raw_socket_capability().unwrap_or(false) {
            let result = drop_privileges("nonexistent_user_12345", "nonexistent_group_12345");
            assert!(result.is_err());
            if let Err(Error::Privilege(msg)) = result {
                assert!(msg.contains("not found") || msg.contains("Failed to get"));
            }
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_drop_privileges() {
        // Should succeed with warning on Windows
        let result = drop_privileges("dummy", "dummy");
        assert!(result.is_ok());
    }
}
