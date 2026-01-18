use std::fmt;
use std::fs;
use std::path::PathBuf;

const SESSION_DIRS: &[&str] = &[
    "/usr/share/wayland-sessions",
    "/usr/share/xsessions",
    "/usr/local/share/wayland-sessions",
    "/usr/local/share/xsessions",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub name: String,
    pub command: Vec<String>,
    pub session_type: SessionType,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn get_sessions() -> Vec<Session> {
    let mut sessions = Vec::new();

    for dir in SESSION_DIRS {
        let session_type = if dir.contains("wayland") {
            SessionType::Wayland
        } else {
            SessionType::X11
        };

        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "desktop") {
                if let Some(session) = parse_desktop_file(&path, session_type) {
                    sessions.push(session);
                }
            }
        }
    }

    sessions.sort_by(|a, b| a.name.cmp(&b.name));
    sessions
}

fn parse_desktop_entry(content: &str, session_type: SessionType) -> Option<Session> {
    let mut name = None;
    let mut exec = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("Name=") {
            name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Exec=") {
            exec = Some(value.to_string());
        }
    }

    let name = name?;
    let exec = exec?;
    let command: Vec<String> = exec.split_whitespace().map(String::from).collect();

    Some(Session {
        name,
        command,
        session_type,
    })
}

fn parse_desktop_file(path: &PathBuf, session_type: SessionType) -> Option<Session> {
    let content = fs::read_to_string(path).ok()?;
    parse_desktop_entry(&content, session_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_desktop_entry() {
        let content = "[Desktop Entry]\nName=Sway\nExec=sway\nType=Application";
        let session = parse_desktop_entry(content, SessionType::Wayland).unwrap();

        assert_eq!(session.name, "Sway");
        assert_eq!(session.command, vec!["sway"]);
        assert_eq!(session.session_type, SessionType::Wayland);
    }

    #[test]
    fn parse_desktop_entry_with_args() {
        let content = "Name=GNOME\nExec=gnome-session --session=gnome";
        let session = parse_desktop_entry(content, SessionType::Wayland).unwrap();

        assert_eq!(session.name, "GNOME");
        assert_eq!(session.command, vec!["gnome-session", "--session=gnome"]);
    }

    #[test]
    fn parse_desktop_entry_missing_name() {
        let content = "Exec=sway";
        assert!(parse_desktop_entry(content, SessionType::Wayland).is_none());
    }

    #[test]
    fn parse_desktop_entry_missing_exec() {
        let content = "Name=Sway";
        assert!(parse_desktop_entry(content, SessionType::Wayland).is_none());
    }

    #[test]
    fn parse_desktop_entry_empty() {
        assert!(parse_desktop_entry("", SessionType::Wayland).is_none());
    }

    #[test]
    fn parse_desktop_entry_with_leading_whitespace() {
        let content = "  Name=Sway\n  Exec=sway";
        let session = parse_desktop_entry(content, SessionType::Wayland).unwrap();

        assert_eq!(session.name, "Sway");
        assert_eq!(session.command, vec!["sway"]);
    }

    #[test]
    fn session_display() {
        let session = Session {
            name: "Sway".to_string(),
            command: vec!["sway".to_string()],
            session_type: SessionType::Wayland,
        };
        assert_eq!(format!("{}", session), "Sway");
    }
}
