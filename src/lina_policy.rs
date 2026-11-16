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

// Two builds come from these sources (selected by LINA_DESK_VARIANT at build
// time, see build.rs):
// - client: the person being helped; incoming only, Allow decides.
// - operator: the owner's helper; outgoing only, nobody can connect to it.

/// Internal app/config name (config folder, IPC names). No spaces on purpose:
/// upstream assumes app names are alphanumeric. Distinct from stock RustDesk
/// so neither build shares RustDesk's folders.
#[cfg(not(lina_operator))]
pub const APP_NAME: &str = "LinaDesk";
#[cfg(lina_operator)]
pub const APP_NAME: &str = "LinaDeskOperator";
/// Name shown to people.
#[cfg(not(lina_operator))]
pub const DISPLAY_NAME: &str = "Lina Desk";
#[cfg(lina_operator)]
pub const DISPLAY_NAME: &str = "Lina Desk Operator";
/// True for the operator helper build.
pub const IS_OPERATOR: bool = cfg!(lina_operator);
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
    // Approved pilot defaults: screen + control + files + clipboard on; the
    // person can switch control/files off before or during the session.
    ("enable-keyboard", "Y"),
    ("enable-file-transfer", "Y"),
    // Text copy/paste both ways during a session, and file copy/paste between
    // the two PCs.
    ("enable-clipboard", "Y"),
    ("enable-file-copy-paste", "Y"),
    // Shell: never part of a normal session. A helper has to send a separate
    // Shell request, which shows its own Allow/Decline card.
    ("enable-terminal", "Y"),
    // Hidden in the pilot.
    ("enable-audio", "N"),
    ("enable-camera", "N"),
    ("enable-tunnel", "N"),
    ("enable-remote-restart", "N"),
    ("enable-record-session", "N"),
    ("enable-block-input", "N"),
    ("enable-privacy-mode", "N"),
    ("enable-remote-printer", "N"),
    ("allow-remote-config-modification", "N"),
    // Permissions come from the switches above, never from a "full access"
    // preset that would turn the always-off ones back on.
    ("access-mode", "custom"),
    // No LAN discovery and no direct IP access. The web socket transport is
    // off unless this run had to fall back to it (see `transport`).
    ("enable-lan-discovery", "N"),
    ("direct-server", "N"),
    ("allow-websocket", "N"),
];

/// Never offered, whatever a saved option, an access preset or the server
/// says: sound, camera, tunnels, remote restart, recording, block input,
/// privacy mode, remote printer.
pub const ALWAYS_OFF: &[&str] = &[
    "enable-audio",
    "enable-camera",
    "enable-tunnel",
    "enable-remote-restart",
    "enable-record-session",
    "enable-block-input",
    "enable-privacy-mode",
    "enable-remote-printer",
];

pub fn is_always_off(option: &str) -> bool {
    ALWAYS_OFF.contains(&option)
}

/// The same list by the names the Allow window uses for its switches.
pub fn is_always_off_switch(name: &str) -> bool {
    matches!(
        name,
        "audio"
            | "camera"
            | "tunnel"
            | "restart"
            | "recording"
            | "block_input"
            | "privacy_mode"
            | "remote_printer"
    )
}

/// Local (per-user UI) options that stay fixed.
const OVERWRITE_LOCAL_SETTINGS: &[(&str, &str)] = &[
    // No upstream update check or auto update.
    ("enable-check-update", "N"),
    ("allow-auto-update", "N"),
    // Direct first: try a direct path over UDP and IPv6 before the relay.
    // Upstream leaves both off for any server that is not its public one.
    ("enable-udp-punch", "Y"),
    ("enable-ipv6-punch", "Y"),
    // No session recording, by hand or automatic.
    ("hide-recording-button", "Y"),
    ("allow-auto-record-outgoing", "N"),
    ("allow-auto-record-incoming", "N"),
    // The approved client look is the dark one; the Lina screens are drawn
    // for it, so the theme is fixed in this release.
    ("theme", "dark"),
];

/// Session defaults a new connection starts from. Lowest precedence: a choice
/// the user saves later (Settings, or the toolbar of one peer) overrides it.
const DEFAULT_DISPLAY_SETTINGS: &[(&str, &str)] = &[
    // Phone screens are far larger than the operator's window, so showing them
    // at their pixel size pushes them outside it. Fit them to the window.
    ("view_style", "adaptive"),
];

/// Operator only: every session shows delay, frame rate and codec, so a slow
/// link is seen at once.
const OPERATOR_DISPLAY_SETTINGS: &[(&str, &str)] = &[("show_quality_monitor", "Y")];

/// Switches read by `is_incoming_only`, `is_disable_*`.
const HARD_SETTINGS: &[(&str, &str)] = &[
    // No `disable-installation` for the client: it installs itself on first
    // run (service included), which is what lets the helper see and use
    // administrator prompts. The operator helper stays portable, see `apply`.
    ("disable-account", "Y"),
    ("disable-ab", "Y"),
    // No `disable-tcp-listen` on purpose: with it the client never tries the
    // intranet address or a punch and every session goes through the relay.
    // The helper tries the direct paths first (src/client.rs no longer forces
    // the relay); the relay is the fallback.
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
        put(&config::DEFAULT_DISPLAY_SETTINGS, DEFAULT_DISPLAY_SETTINGS);
        put(&config::HARD_SETTINGS, HARD_SETTINGS);
        if IS_OPERATOR {
            // Operator helper: connects out only; it never registers for
            // incoming sessions, so this PC/phone cannot be controlled
            // through it.
            put(&config::HARD_SETTINGS, &[("conn-type", "outgoing")]);
            // The Lina app starts the helper as a plain program: no installer,
            // no service.
            put(&config::HARD_SETTINGS, &[("disable-installation", "Y")]);
            put(&config::DEFAULT_DISPLAY_SETTINGS, OPERATOR_DISPLAY_SETTINGS);
        } else {
            // Client: only receives help; no "connect to another PC".
            put(&config::HARD_SETTINGS, &[("conn-type", "incoming")]);
        }
        put(&config::BUILTIN_SETTINGS, BUILTIN_SETTINGS);
        install_panic_hook();
    });
}

/// Release builds abort on a panic and leave nothing behind. Write what
/// panicked and where into the log (and a plain `crash.txt` beside it, for a
/// process whose logger was not running yet) before the process goes away.
fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_owned());
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_owned()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "no message".to_owned()
        };
        let thread = std::thread::current();
        let line = format!(
            "PANIC {} {} thread '{}' at {}: {}",
            DISPLAY_NAME,
            crate::VERSION,
            thread.name().unwrap_or("unnamed"),
            location,
            message
        );
        hbb_common::log::error!("{}", line);
        hbb_common::log::logger().flush();
        write_crash_line(&line);
        previous(info);
    }));
}

fn write_crash_line(line: &str) {
    use std::io::Write;
    let dir = config::Config::log_path();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("crash.txt");
    // Keep the file small: start over once it passes 256 KB.
    if std::fs::metadata(&path).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
        let _ = std::fs::remove_file(&path);
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(
            file,
            "{} {}",
            hbb_common::chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            line
        );
    }
}

/// How this device reaches the server when the network blocks the normal way.
///
/// A device first registers over UDP, the fastest way in and the one direct
/// connections need. Networks that drop UDP get TCP on the same port; networks
/// that only let web traffic out get a secure web socket on port 443 (the
/// server's web front door forwards `/ws/id` and `/ws/relay`). Each step is
/// taken only after the one before it failed in this run, and a dropped link
/// starts again from the top, so a laptop that leaves the office goes back to
/// the fast path by itself.
pub mod transport {
    use super::{config, SERVER_HOST};
    use std::sync::atomic::{AtomicU8, Ordering};

    pub const UDP: u8 = 0;
    pub const TCP: u8 = 1;
    pub const WEB: u8 = 2;

    static LEVEL: AtomicU8 = AtomicU8::new(UDP);

    pub fn level() -> u8 {
        LEVEL.load(Ordering::SeqCst)
    }

    fn set(level: u8) {
        LEVEL.store(level, Ordering::SeqCst);
        let mut settings = config::OVERWRITE_SETTINGS.write().unwrap();
        if level == WEB {
            settings.insert("allow-websocket".to_owned(), "Y".to_owned());
            // An https api server is what makes the web socket address secure
            // (wss on 443) instead of plain.
            settings.insert("api-server".to_owned(), format!("https://{SERVER_HOST}"));
        } else {
            settings.insert("allow-websocket".to_owned(), "N".to_owned());
            settings.remove("api-server");
        }
    }

    /// The current way in failed: take the next one. After the last one, start
    /// again from the top. Returns the new level.
    pub fn step_down() -> u8 {
        let next = match level() {
            UDP => TCP,
            TCP => WEB,
            _ => UDP,
        };
        set(next);
        hbb_common::log::info!("server link: switching to {}", name(next));
        next
    }

    /// Back to the fastest way in.
    pub fn reset() {
        if level() != UDP {
            set(UDP);
            hbb_common::log::info!("server link: back to {}", name(UDP));
        }
    }

    pub fn name(level: u8) -> &'static str {
        match level {
            UDP => "udp",
            TCP => "tcp",
            _ => "web socket on 443",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin(key: &str) -> String {
        config::BUILTIN_SETTINGS
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    fn overwritten_display_setting(key: &str) -> String {
        config::OVERWRITE_DISPLAY_SETTINGS
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    #[test]
    fn policy_opens_admin_shell_and_clipboard() {
        apply();
        for key in [
            "enable-keyboard",
            "enable-file-transfer",
            "enable-terminal",
            "enable-clipboard",
            "enable-file-copy-paste",
        ] {
            assert_eq!(config::Config::get_option(key), "Y", "{key}");
        }
        for key in [
            "enable-audio",
            "enable-camera",
            "enable-tunnel",
            "enable-remote-restart",
            "enable-record-session",
            "enable-block-input",
            "enable-privacy-mode",
            "enable-remote-printer",
        ] {
            assert_eq!(config::Config::get_option(key), "N", "{key}");
        }
        // The helper's admin ask reaches the person instead of being hidden away.
        assert_ne!(builtin("hide-elevate-button-in-accept-window"), "Y");
        // Neither build forces clipboard off in its own sessions.
        assert_ne!(overwritten_display_setting("disable_clipboard"), "Y");
    }

    #[test]
    fn policy_fixes_server_key_and_click_approval() {
        apply();
        assert_eq!(config::Config::get_option("custom-rendezvous-server"), SERVER_HOST);
        assert_eq!(config::Config::get_option("key"), SERVER_PUBLIC_KEY);
        assert_eq!(config::Config::get_option("approve-mode"), "click");
        config::Config::set_option("key".to_owned(), "other".to_owned());
        assert_eq!(config::Config::get_option("key"), SERVER_PUBLIC_KEY);
        assert_eq!(config::is_incoming_only(), !IS_OPERATOR);
        assert_eq!(config::is_outgoing_only(), IS_OPERATOR);
        assert_eq!(crate::get_app_name(), APP_NAME);
    }

    #[test]
    fn always_off_survives_a_full_access_preset() {
        apply();
        config::Config::set_option("access-mode".to_owned(), "full".to_owned());
        assert_eq!(config::Config::get_option("access-mode"), "custom");
        for key in ALWAYS_OFF {
            assert_eq!(config::Config::get_option(key), "N", "{key}");
            assert!(is_always_off(key));
        }
        assert!(!is_always_off("enable-keyboard"));
        assert!(is_always_off_switch("audio"));
        assert!(!is_always_off_switch("keyboard"));
    }

    #[test]
    fn client_installs_and_operator_stays_portable() {
        apply();
        assert_eq!(config::is_disable_installation(), IS_OPERATOR);
    }

    #[test]
    fn server_link_falls_back_step_by_step_and_recovers() {
        apply();
        transport::reset();
        assert_eq!(transport::level(), transport::UDP);
        assert!(!config::use_ws());
        assert_eq!(transport::step_down(), transport::TCP);
        assert!(!config::use_ws());
        assert_eq!(transport::step_down(), transport::WEB);
        assert!(config::use_ws());
        assert_eq!(
            config::Config::get_option("api-server"),
            format!("https://{SERVER_HOST}")
        );
        // Nothing left to try: start again from the fastest way in.
        assert_eq!(transport::step_down(), transport::UDP);
        assert!(!config::use_ws());
        assert_eq!(config::Config::get_option("api-server"), "");
    }

    #[test]
    fn policy_connects_directly_and_fits_new_sessions() {
        apply();
        // The relay is the fallback, not the only route.
        assert!(!config::is_disable_tcp_listen());
        assert_eq!(config::LocalConfig::get_option("enable-udp-punch"), "Y");
        assert_eq!(config::LocalConfig::get_option("enable-ipv6-punch"), "Y");
        assert_eq!(config::Config::get_option("direct-server"), "N");
        assert_eq!(config::Config::get_option("enable-lan-discovery"), "N");
        // A new session fits the window; a saved user choice still wins.
        let mut defaults = config::UserDefaultConfig::load();
        defaults.set("view_style".to_owned(), String::new());
        assert_eq!(config::UserDefaultConfig::load().get("view_style"), "adaptive");
        defaults.set("view_style".to_owned(), "original".to_owned());
        assert_eq!(config::UserDefaultConfig::load().get("view_style"), "original");
        defaults.set("view_style".to_owned(), "adaptive".to_owned());
        assert_eq!(config::UserDefaultConfig::load().get("view_style"), "adaptive");
    }
}
