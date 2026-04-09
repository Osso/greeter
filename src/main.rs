mod config;
mod greetd;
mod sessions;
mod theme;
mod users;

use iced::keyboard::{self, Event as KeyEvent, Key, key};
use iced::widget::{Id, button, column, container, pick_list, text, text_input};
use iced::{Center, Element, Fill, Font, Subscription, Task, widget::operation};
use tracing::{error, info};

use config::Config;
use greetd::AuthStatus;
use sessions::Session;

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

struct Greeter {
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
        let user_count = users::get_usernames().len();
        let sessions = sessions::get_sessions();

        let default_session = config
            .default_session
            .as_ref()
            .and_then(|name| sessions.iter().find(|s| s.name.eq_ignore_ascii_case(name)))
            .cloned()
            .or_else(|| sessions.first().cloned());

        let username = config.default_user.unwrap_or_default();

        info!("Found {} users, {} sessions", user_count, sessions.len());

        (
            Self {
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
            Message::UsernameChanged(username) => self.update_username(username),
            Message::PasswordChanged(password) => self.update_password(password),
            Message::SessionSelected(session) => self.select_session(session),
            Message::Submit => self.submit(),
            Message::AuthResult(result) => self.handle_auth_result(result),
            Message::SessionStarted(result) => self.handle_session_started(result),
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

    fn update_username(&mut self, username: String) -> Task<Message> {
        self.username = username;
        Task::none()
    }

    fn update_password(&mut self, password: String) -> Task<Message> {
        self.password = password;
        Task::none()
    }

    fn select_session(&mut self, session: Session) -> Task<Message> {
        self.selected_session = Some(session);
        Task::none()
    }

    fn submit(&mut self) -> Task<Message> {
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

    fn handle_auth_result(&mut self, result: Result<AuthStatus, String>) -> Task<Message> {
        match result {
            Ok(AuthStatus::Success) => self.start_selected_session(),
            Ok(AuthStatus::AuthRequired) => self.finish_auth_attempt("Authentication required"),
            Err(error) => self.fail_auth_attempt(error),
        }
    }

    fn start_selected_session(&mut self) -> Task<Message> {
        self.status = "Starting session...".to_string();
        let session = self.selected_session.clone();
        Task::perform(
            async move { greetd::start_session(session).await },
            Message::SessionStarted,
        )
    }

    fn finish_auth_attempt(&mut self, status: &str) -> Task<Message> {
        self.status = status.to_string();
        self.reset_auth_state();
        Task::none()
    }

    fn fail_auth_attempt(&mut self, error_message: String) -> Task<Message> {
        error!("Auth failed: {}", error_message);
        self.status = format!("Error: {}", error_message);
        self.reset_auth_state();
        Task::none()
    }

    fn handle_session_started(&mut self, result: Result<(), String>) -> Task<Message> {
        match result {
            Ok(()) => {
                self.status = "Session started!".to_string();
                std::process::exit(0);
            }
            Err(error_message) => self.fail_session_start(error_message),
        }
    }

    fn fail_session_start(&mut self, error_message: String) -> Task<Message> {
        error!("Session start failed: {}", error_message);
        self.status = format!("Error: {}", error_message);
        self.authenticating = false;
        Task::none()
    }

    fn reset_auth_state(&mut self) {
        self.authenticating = false;
        self.password.clear();
    }

    #[cfg(test)]
    fn test_new() -> Self {
        Self {
            sessions: vec![],
            username: String::new(),
            password: String::new(),
            selected_session: None,
            status: String::new(),
            authenticating: false,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let card = container(self.form()).style(theme::card);

        container(card)
            .width(Fill)
            .height(Fill)
            .align_x(Center)
            .align_y(Center)
            .style(theme::background)
            .into()
    }

    fn form(&self) -> Element<'_, Message> {
        column![
            self.username_input(),
            self.password_input(),
            self.session_picker(),
            self.submit_button(),
            self.status_text(),
        ]
        .spacing(16)
        .padding(32)
        .width(320)
        .into()
    }

    fn username_input(&self) -> Element<'_, Message> {
        text_input("Username", &self.username)
            .on_input(Message::UsernameChanged)
            .on_submit(Message::Submit)
            .padding(12)
            .size(16)
            .style(theme::text_input_style)
            .into()
    }

    fn password_input(&self) -> Element<'_, Message> {
        text_input("Password", &self.password)
            .id(Id::new(PASSWORD_INPUT_ID))
            .on_input(Message::PasswordChanged)
            .on_submit(Message::Submit)
            .secure(true)
            .padding(12)
            .size(16)
            .style(theme::text_input_style)
            .into()
    }

    fn session_picker(&self) -> Element<'_, Message> {
        pick_list(
            &self.sessions[..],
            self.selected_session.clone(),
            Message::SessionSelected,
        )
        .placeholder("Select session")
        .padding(12)
        .text_size(16)
        .style(theme::pick_list_style)
        .into()
    }

    fn submit_button(&self) -> Element<'_, Message> {
        button(text("Login").size(16))
            .on_press_maybe((!self.authenticating).then_some(Message::Submit))
            .padding([12, 24])
            .style(theme::button_style)
            .into()
    }

    fn status_text(&self) -> Element<'_, Message> {
        text(&self.status).size(14).style(theme::status_text).into()
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
