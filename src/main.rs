mod config;
mod greetd;
mod sessions;
mod theme;
mod users;

use iced::keyboard::{self, key, Event as KeyEvent, Key};
use iced::widget::{button, column, container, pick_list, text, text_input, Id};
use iced::{Center, Element, Fill, Font, Subscription, Task, widget::operation};
use tracing::{error, info};

use config::Config;
use greetd::AuthStatus;
use sessions::Session;
use users::User;

const PASSWORD_INPUT_ID: &str = "password";

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Greeter::new, Greeter::update, Greeter::view)
        .subscription(Greeter::subscription)
        .title("Greeter")
        .window_size((450.0, 350.0))
        .default_font(Font::with_name("Cantarell"))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    UsernameChanged(String),
    PasswordChanged(String),
    SessionSelected(Session),
    Submit,
    AuthResult(Result<AuthStatus, String>),
    SessionStarted(Result<(), String>),
    FocusNext,
    FocusPrevious,
    KeyboardEvent,
}

#[allow(dead_code)]
struct Greeter {
    users: Vec<User>,
    sessions: Vec<Session>,
    username: String,
    password: String,
    selected_session: Option<Session>,
    status: String,
    authenticating: bool,
}

impl Greeter {
    fn new() -> (Self, Task<Message>) {
        let config = Config::load();
        let users = users::get_users();
        let sessions = sessions::get_sessions();

        let default_session = config
            .default_session
            .as_ref()
            .and_then(|name| sessions.iter().find(|s| s.name.eq_ignore_ascii_case(name)))
            .cloned()
            .or_else(|| sessions.first().cloned());

        let username = config.default_user.unwrap_or_default();

        info!("Found {} users, {} sessions", users.len(), sessions.len());

        (
            Self {
                users,
                sessions,
                username,
                password: String::new(),
                selected_session: default_session,
                status: String::new(),
                authenticating: false,
            },
            operation::focus(Id::new(PASSWORD_INPUT_ID)),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UsernameChanged(username) => {
                self.username = username;
                Task::none()
            }
            Message::PasswordChanged(password) => {
                self.password = password;
                Task::none()
            }
            Message::SessionSelected(session) => {
                self.selected_session = Some(session);
                Task::none()
            }
            Message::Submit => {
                if self.username.is_empty() {
                    self.status = "Enter a username".to_string();
                    return Task::none();
                }

                self.authenticating = true;
                self.status = "Authenticating...".to_string();

                let username = self.username.clone();
                let password = self.password.clone();

                Task::perform(
                    async move { greetd::authenticate(&username, &password).await },
                    Message::AuthResult,
                )
            }
            Message::AuthResult(result) => {
                match result {
                    Ok(AuthStatus::Success) => {
                        self.status = "Starting session...".to_string();
                        let session = self.selected_session.clone();
                        return Task::perform(
                            async move { greetd::start_session(session).await },
                            Message::SessionStarted,
                        );
                    }
                    Ok(AuthStatus::AuthRequired) => {
                        self.status = "Authentication required".to_string();
                    }
                    Err(e) => {
                        error!("Auth failed: {}", e);
                        self.status = format!("Error: {}", e);
                    }
                }
                self.authenticating = false;
                self.password.clear();
                Task::none()
            }
            Message::SessionStarted(result) => {
                match result {
                    Ok(()) => {
                        self.status = "Session started!".to_string();
                        std::process::exit(0);
                    }
                    Err(e) => {
                        error!("Session start failed: {}", e);
                        self.status = format!("Error: {}", e);
                    }
                }
                self.authenticating = false;
                Task::none()
            }
            Message::FocusNext => operation::focus_next(),
            Message::FocusPrevious => operation::focus_previous(),
            Message::KeyboardEvent => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| match event {
            KeyEvent::KeyPressed {
                key: Key::Named(key::Named::Tab),
                modifiers,
                ..
            } => {
                if modifiers.shift() {
                    Message::FocusPrevious
                } else {
                    Message::FocusNext
                }
            }
            _ => Message::KeyboardEvent,
        })
    }

#[cfg(test)]
    fn test_new() -> Self {
        Self {
            users: vec![],
            sessions: vec![],
            username: String::new(),
            password: String::new(),
            selected_session: None,
            status: String::new(),
            authenticating: false,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let username_input = text_input("Username", &self.username)
            .on_input(Message::UsernameChanged)
            .on_submit(Message::Submit)
            .padding(12)
            .size(16)
            .style(theme::text_input_style);

        let password_input = text_input("Password", &self.password)
            .id(Id::new(PASSWORD_INPUT_ID))
            .on_input(Message::PasswordChanged)
            .on_submit(Message::Submit)
            .secure(true)
            .padding(12)
            .size(16)
            .style(theme::text_input_style);

        let session_picker = pick_list(
            &self.sessions[..],
            self.selected_session.clone(),
            Message::SessionSelected,
        )
        .placeholder("Select session")
        .padding(12)
        .text_size(16)
        .style(theme::pick_list_style);

        let submit_button = button(text("Login").size(16))
            .on_press_maybe((!self.authenticating).then_some(Message::Submit))
            .padding([12, 24])
            .style(theme::button_style);

        let status_text = text(&self.status)
            .size(14)
            .style(theme::status_text);

        let card = container(
            column![
                username_input,
                password_input,
                session_picker,
                submit_button,
                status_text,
            ]
            .spacing(16)
            .padding(32)
            .width(320),
        )
        .style(theme::card);

        container(card)
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(Center)
            .style(theme::background)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_changed_updates_state() {
        let mut greeter = Greeter::test_new();
        let _ = greeter.update(Message::UsernameChanged("alice".to_string()));

        assert_eq!(greeter.username, "alice");
    }

    #[test]
    fn password_changed_updates_state() {
        let mut greeter = Greeter::test_new();
        let _ = greeter.update(Message::PasswordChanged("secret".to_string()));

        assert_eq!(greeter.password, "secret");
    }

    #[test]
    fn session_selected_updates_state() {
        let mut greeter = Greeter::test_new();
        let session = Session {
            name: "Sway".to_string(),
            command: vec!["sway".to_string()],
            session_type: sessions::SessionType::Wayland,
        };
        let _ = greeter.update(Message::SessionSelected(session.clone()));

        assert_eq!(greeter.selected_session, Some(session));
    }

    #[test]
    fn submit_without_username_shows_error() {
        let mut greeter = Greeter::test_new();
        let _ = greeter.update(Message::Submit);

        assert_eq!(greeter.status, "Enter a username");
        assert!(!greeter.authenticating);
    }

    #[test]
    fn submit_with_username_starts_auth() {
        let mut greeter = Greeter::test_new();
        greeter.username = "alice".to_string();
        let _ = greeter.update(Message::Submit);

        assert!(greeter.authenticating);
        assert_eq!(greeter.status, "Authenticating...");
    }

    #[test]
    fn auth_error_clears_password() {
        let mut greeter = Greeter::test_new();
        greeter.password = "secret".to_string();
        greeter.authenticating = true;
        let _ = greeter.update(Message::AuthResult(Err("Invalid".to_string())));

        assert!(greeter.password.is_empty());
        assert!(!greeter.authenticating);
        assert!(greeter.status.contains("Invalid"));
    }

    #[test]
    fn auth_required_clears_password() {
        let mut greeter = Greeter::test_new();
        greeter.password = "secret".to_string();
        greeter.authenticating = true;
        let _ = greeter.update(Message::AuthResult(Ok(AuthStatus::AuthRequired)));

        assert!(greeter.password.is_empty());
        assert!(!greeter.authenticating);
    }
}
