//! Lina Desk client policy.
//!
//! Lina Desk is a human-operated support client: the helper connects by ID,
//! the person at this device sees the request and presses Allow. This module
//! is the single place where the shipped build fixes its server, key and
//! permission defaults. It replaces RustDesk's external `custom.txt`
//! mechanism: nothing read from disk, the executable name or the command line
//! can change these values in this build.
//!
//! The values are written into the stock hbb_common setting maps, so the
//! existing precedence rules apply: OVERWRITE_* values win over saved options
//! and are never persisted as user overrides, HARD_SETTINGS drive the
//! `is_incoming_only`/`is_disable_*` switches, BUILTIN_SETTINGS hide UI.

use hbb_common::config;
use std::sync::Once;

/// Internal app/config name (config folder, IPC names). No spaces on purpose:
/// upstream assumes app names are alphanumeric.
pub const APP_NAME: &str = "LinaDesk";
/// Name shown to people.
pub const DISPLAY_NAME: &str = "Lina Desk";
/// Legal business name shown on the client.
pub const COMPANY_NAME: &str = "Xcode technologies SRL";
/// The only rendezvous/relay host this build talks to.
pub const SERVER_HOST: &str = "desk.xcode.it.com";
/// Public key of that server.
pub const SERVER_PUBLIC_KEY: &str = "SiKlUxIe00QSAPd1JXGLvb4Vxx8W4mxx4HnfqKWIqQI=";

/// Server options fixed for every session (Config::get_option reads these first).
const OVERWRITE_SETTINGS: &[(&str, &str)] = &[
    // Own server only, no public-server fallback.
    ("custom-rendezvous-server", SERVER_HOST),
    ("relay-server", SERVER_HOST),
    ("key", SERVER_PUBLIC_KEY),
    // Click-to-allow only: no permanent or one-time password access.
    ("approve-mode", "click"),
    ("verification-method", "use-temporary-password"),
    // Approved pilot defaults: screen + control + files on; the person can
    // switch control/files off before or during the session.
    ("enable-keyboard", "Y"),
    ("enable-file-transfer", "Y"),
    // Shell: never part of a normal session. A helper has to send a separate
    // Shell request, which shows its own Allow/Decline card.
    ("enable-terminal", "Y"),
    // Hidden in the pilot.
    ("enable-clipboard", "N"),
    ("enable-file-copy-paste", "N"),
    ("enable-audio", "N"),
    ("enable-camera", "N"),
    ("enable-tunnel", "N"),
    ("enable-remote-restart", "N"),
    ("enable-record-session", "N"),
    ("enable-block-input", "N"),
    ("enable-privacy-mode", "N"),
    ("enable-remote-printer", "N"),
    ("allow-remote-config-modification", "N"),
    // No LAN discovery, no direct IP access, no web socket transport.
    ("enable-lan-discovery", "N"),
    ("direct-server", "N"),
    ("allow-websocket", "N"),
];

/// Local (per-user UI) options that stay fixed.
const OVERWRITE_LOCAL_SETTINGS: &[(&str, &str)] = &[
    // No upstream update check or auto update.
    ("enable-check-update", "N"),
    ("allow-auto-update", "N"),
    // The approved client look is the dark one; the Lina screens are drawn
    // for it, so the theme is fixed in this release.
    ("theme", "dark"),
];

/// Switches read by `is_incoming_only`, `is_disable_*`.
const HARD_SETTINGS: &[(&str, &str)] = &[
    // This device only receives help; there is no "connect to another PC".
    ("conn-type", "incoming"),
    // Portable first: no installer, no service.
    ("disable-installation", "Y"),
    ("disable-account", "Y"),
    ("disable-ab", "Y"),
    ("disable-tcp-listen", "Y"),
    // No server/security settings and no command-line option changes.
    ("disable-settings", "Y"),
];

/// UI switches.
const BUILTIN_SETTINGS: &[(&str, &str)] = &[
    ("hide-security-settings", "Y"),
    ("hide-network-settings", "Y"),
    ("hide-server-settings", "Y"),
    ("hide-proxy-settings", "Y"),
    ("hide-websocket-settings", "Y"),
    ("hide-remote-printer-settings", "Y"),
    ("hide-stop-service", "Y"),
    ("hide-powered-by-me", "Y"),
    ("hide-elevate-button-in-accept-window", "Y"),
    ("disable-change-permanent-password", "Y"),
    ("disable-change-id", "Y"),
    ("disable-unlock-pin", "Y"),
    ("allow-deep-link-password", "N"),
    ("allow-deep-link-server-settings", "N"),
    ("allow-logon-screen-password", "N"),
    ("allow-command-line-settings-when-settings-disabled", "N"),
];

static APPLY: Once = Once::new();

fn put(map: &std::sync::RwLock<std::collections::HashMap<String, String>>, values: &[(&str, &str)]) {
    let mut map = map.write().unwrap();
    for (k, v) in values {
        map.insert((*k).to_owned(), (*v).to_owned());
    }
}

/// Apply the fixed Lina Desk policy. Safe to call more than once.
pub fn apply() {
    APPLY.call_once(|| {
        *config::APP_NAME.write().unwrap() = APP_NAME.to_owned();
        *config::PROD_RENDEZVOUS_SERVER.write().unwrap() = SERVER_HOST.to_owned();
        // Never take a server from the executable name.
        config::EXE_RENDEZVOUS_SERVER.write().unwrap().clear();
        put(&config::OVERWRITE_SETTINGS, OVERWRITE_SETTINGS);
        put(&config::OVERWRITE_LOCAL_SETTINGS, OVERWRITE_LOCAL_SETTINGS);
        put(&config::HARD_SETTINGS, HARD_SETTINGS);
        put(&config::BUILTIN_SETTINGS, BUILTIN_SETTINGS);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_fixes_server_key_and_click_approval() {
        apply();
        assert_eq!(config::Config::get_option("custom-rendezvous-server"), SERVER_HOST);
        assert_eq!(config::Config::get_option("key"), SERVER_PUBLIC_KEY);
        assert_eq!(config::Config::get_option("approve-mode"), "click");
        config::Config::set_option("key".to_owned(), "other".to_owned());
        assert_eq!(config::Config::get_option("key"), SERVER_PUBLIC_KEY);
        assert!(config::is_incoming_only());
        assert_eq!(crate::get_app_name(), APP_NAME);
    }
}
