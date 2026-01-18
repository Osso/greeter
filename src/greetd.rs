use std::env;

use greetd_ipc::codec::TokioCodec;
use greetd_ipc::{Request, Response};
use tokio::net::UnixStream;
use tracing::info;

use crate::sessions::Session;

const GREETD_SOCK: &str = "GREETD_SOCK";
const FAKE_PASSWORD: &str = "test";

fn is_fake_mode() -> bool {
    env::var(GREETD_SOCK).is_err()
}

#[derive(Debug, Clone)]
pub enum AuthStatus {
    Success,
    AuthRequired,
}

pub struct GreetdClient {
    socket: UnixStream,
}

impl GreetdClient {
    pub async fn connect() -> Result<Self, String> {
        let sock_path =
            env::var(GREETD_SOCK).map_err(|_| format!("Missing {GREETD_SOCK} env var"))?;
        let socket = UnixStream::connect(&sock_path)
            .await
            .map_err(|e| format!("Failed to connect to greetd: {e}"))?;
        Ok(Self { socket })
    }

    async fn send(&mut self, request: Request) -> Result<Response, String> {
        request
            .write_to(&mut self.socket)
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;
        Response::read_from(&mut self.socket)
            .await
            .map_err(|e| format!("Failed to read response: {e}"))
    }

    pub async fn create_session(&mut self, username: &str) -> Result<Response, String> {
        info!("Creating session for {username}");
        self.send(Request::CreateSession {
            username: username.to_string(),
        })
        .await
    }

    pub async fn post_auth(&mut self, response: Option<String>) -> Result<Response, String> {
        info!("Posting auth response");
        self.send(Request::PostAuthMessageResponse { response }).await
    }

    pub async fn start_session(&mut self, cmd: Vec<String>) -> Result<Response, String> {
        info!("Starting session with command: {cmd:?}");
        self.send(Request::StartSession {
            cmd,
            env: Vec::new(),
        })
        .await
    }

    pub async fn cancel_session(&mut self) -> Result<Response, String> {
        info!("Cancelling session");
        self.send(Request::CancelSession).await
    }
}

pub async fn authenticate(username: &str, password: &str) -> Result<AuthStatus, String> {
    if is_fake_mode() {
        info!("[FAKE] Authenticating {username}");
        return if password == FAKE_PASSWORD {
            Ok(AuthStatus::Success)
        } else {
            Err("Invalid password (use 'test' in fake mode)".to_string())
        };
    }

    let mut client = GreetdClient::connect().await?;

    let response = client.create_session(username).await?;

    match response {
        Response::Success => Ok(AuthStatus::Success),
        Response::AuthMessage { .. } => {
            let response = client.post_auth(Some(password.to_string())).await?;
            match response {
                Response::Success => Ok(AuthStatus::Success),
                Response::AuthMessage { .. } => Ok(AuthStatus::AuthRequired),
                Response::Error { description, .. } => {
                    let _ = client.cancel_session().await;
                    Err(description)
                }
            }
        }
        Response::Error { description, .. } => Err(description),
    }
}

pub async fn start_session(session: Option<Session>) -> Result<(), String> {
    let session = session.ok_or("No session selected")?;

    if is_fake_mode() {
        info!("[FAKE] Would start session: {:?}", session.command);
        return Ok(());
    }

    let mut client = GreetdClient::connect().await?;

    let response = client.start_session(session.command).await?;
    match response {
        Response::Success => Ok(()),
        Response::Error { description, .. } => Err(description),
        Response::AuthMessage { .. } => Err("Unexpected auth message".to_string()),
    }
}
