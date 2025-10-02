// Debug session management
//
// Manages JDWP connection state, breakpoints, and thread tracking

use jdwp_client::JdwpConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SessionId = String;

#[derive(Debug)]
pub struct DebugSession {
    pub connection: JdwpConnection,
    pub breakpoints: HashMap<String, BreakpointInfo>,
    pub threads: HashMap<String, ThreadInfo>,
}

#[derive(Debug, Clone)]
pub struct BreakpointInfo {
    pub id: String,
    pub request_id: i32,
    pub class_pattern: String,
    pub line: u32,
    pub method: Option<String>,
    pub enabled: bool,
    pub hit_count: u32,
}

#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub suspended: bool,
}

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<SessionId, Arc<Mutex<DebugSession>>>>>,
    current_session: Arc<Mutex<Option<SessionId>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn create_session(&self, connection: JdwpConnection) -> SessionId {
        let session_id = format!("session_{}", uuid::v4());
        let session = DebugSession {
            connection,
            breakpoints: HashMap::new(),
            threads: HashMap::new(),
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), Arc::new(Mutex::new(session)));

        // Set as current session
        let mut current = self.current_session.lock().await;
        *current = Some(session_id.clone());

        session_id
    }

    pub async fn get_current_session(&self) -> Option<Arc<Mutex<DebugSession>>> {
        let current = self.current_session.lock().await;
        if let Some(session_id) = current.as_ref() {
            let sessions = self.sessions.lock().await;
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }

    pub async fn get_current_session_id(&self) -> Option<SessionId> {
        let current = self.current_session.lock().await;
        current.clone()
    }

    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(session_id);

        // Clear current if it was this session
        let mut current = self.current_session.lock().await;
        if current.as_ref() == Some(&session_id.to_string()) {
            *current = None;
        }
    }
}

// Simple UUID generation for session IDs
mod uuid {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);

    pub fn v4() -> String {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        format!("{:x}{:x}", timestamp, counter)
    }
}
