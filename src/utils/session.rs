// Importing necessary modules and structures from the `rocket` and `uuid` crates.
use rocket::http::CookieJar;
use rocket::State;
use uuid::{Uuid};

// Importing the `SessionHandler` and `Session` structures from the `structs` module.
pub use structs::{SessionHandler, Session};

// Importing the `set_session_key`, `get_session_key`, and `remove_session_key` functions from the `cookies` module.
use cookies::{set_session_key, get_session_key, remove_session_key};

// Importing the `Game` structure from the `game` module in `utils`.
use crate::utils::game::Game;

// Constant representing the session key reference.
const SESSION_KEY_REF: &'static str = "session_key";

// Type alias for `Game`.
type T = Game;

// The `structs` module contains the `SessionHandler` and `Session` structures.
mod structs {
    // Importing necessary modules and structures from the `std`, `tokio`, `uuid`, and `crate` crates.
    use tokio::sync::{Mutex, MutexGuard, RwLock};
    use std::collections::HashMap;
    use crate::utils::session::T;
    use std::sync::Arc;
    use uuid::Uuid;

    // The `SessionHandler` structure represents a session handler that manages multiple sessions.
    pub struct SessionHandler {
        // A thread-safe, mutable map of session IDs to sessions.
        sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    }

    impl SessionHandler {
        // Method to create a new `SessionHandler`.
        pub fn new() -> Self {
            SessionHandler {
                sessions: Arc::new(RwLock::new(HashMap::new()))
            }
        }

        // Asynchronous method to get a session by its ID.
        pub async fn get(&self, id: Uuid) -> Option<Session> {
            let sessions = &self.sessions.read().await;
            sessions.get(&id).cloned()
        }

        // Asynchronous method to add a session with a given ID.
        pub async fn add(&self, id: Uuid, session: Session) {
            let sessions = &mut self.sessions.write().await;
            sessions.insert(id, session);
        }

        // Asynchronous method to remove a session by its ID.
        pub async fn remove(&self, id: Uuid) {
            self.sessions.write().await.remove(&id);
        }
    }

    // The `Session` structure represents a session that contains a game state.
    #[derive(Clone)]
    pub struct Session {
        // A thread-safe, mutable game state.
        state: Arc<Mutex<T>>,
    }

    impl Session {
        // Method to create a new `Session` with a given game state.
        pub fn new(o: T) -> Self {
            Session {
                state: Arc::new(Mutex::new(o))
            }
        }

        // Asynchronous method to get the game state of the session.
        pub async fn get(&self) -> MutexGuard<T> {
            self.state.lock().await
        }
    }
}

// The `cookies` module contains functions for managing session cookies.
mod cookies {
    // Importing necessary modules and structures from the `rocket` and `uuid` crates.
    use crate::utils::session::SESSION_KEY_REF;
    use rocket::http::{Cookie, CookieJar};
    use uuid::Uuid;

    // Function to get the session key from a cookie jar.
    pub fn get_session_key(cookie_jar: &CookieJar<'_>) -> Option<Uuid> {
        let session_key = cookie_jar.get_private(SESSION_KEY_REF)?.value().to_string();
        Uuid::parse_str(&session_key).ok()
    }

    // Function to set the session key in a cookie jar.
    pub fn set_session_key(cookie_jar: &CookieJar<'_>, uuid: Uuid) {
        cookie_jar.add_private(Cookie::new(SESSION_KEY_REF, uuid.to_string()))
    }

    // Function to remove the session key from a cookie jar.
    pub fn remove_session_key(cookie_jar: &CookieJar<'_>) {
        cookie_jar.remove_private(SESSION_KEY_REF);
    }
}

// Asynchronous function to add a session with a given game state to a session handler and set the session key in a cookie jar.
pub async fn add_session(state: T, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) {
    let id = Uuid::new_v4();
    let session = Session::new(state);
    session_handler.add(id.clone(), session).await;
    set_session_key(cookie_jar, id);
}

// Asynchronous function to find a session in a session handler by the session key in a cookie jar.
pub async fn find_session(cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Option<Session> {
    let session_key = get_session_key(cookie_jar)?;
    session_handler.get(session_key).await
}

// Asynchronous function to remove a session from a session handler by the session key in a cookie jar and remove the session key from the cookie jar.
pub async fn remove_session(cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) {
    let key = get_session_key(cookie_jar);
    match key {
        None => {}
        Some(key) => {
            session_handler.remove(key).await
        }
    }
    remove_session_key(cookie_jar);
}