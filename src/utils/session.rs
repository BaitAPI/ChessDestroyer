use rocket::http::CookieJar;
use rocket::State;
use uuid::{Uuid};
pub use structs::{SessionHandler, Session};
use cookies::{set_session_key, get_session_key, remove_session_key};
use crate::utils::game::Game;

const SESSION_KEY_REF: &'static str = "session_key";

type T = Game;

mod structs {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::{Mutex, MutexGuard, RwLock};
    use uuid::Uuid;
    use crate::utils::session::T;

    pub struct SessionHandler {
        sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    }

    impl SessionHandler {
        pub fn new() -> Self {
            SessionHandler {
                sessions: Arc::new(RwLock::new(HashMap::new()))
            }
        }
        pub async fn get(&self, id: Uuid) -> Option<Session> {
            let sessions = &self.sessions.read().await;
            sessions.get(&id).cloned()
        }
        pub async fn add(&self, id: Uuid, session: Session) {
            let sessions = &mut self.sessions.write().await;
            sessions.insert(id, session);
        }
        pub async fn remove(&self, id: Uuid) {
            self.sessions.write().await.remove(&id);
        }
    }

    #[derive(Clone)]
    pub struct Session {
        state: Arc<Mutex<T>>,
    }

    impl Session {
        pub fn new(o: T) -> Self {
            Session {
                state: Arc::new(Mutex::new(o))
            }
        }
        pub async fn get(&self) -> MutexGuard<T> {
            self.state.lock().await
        }
    }
}

mod cookies {
    use rocket::http::{Cookie, CookieJar};
    use uuid::Uuid;
    use crate::utils::session::SESSION_KEY_REF;

    pub fn get_session_key(cookie_jar: &CookieJar<'_>) -> Option<Uuid> {
        let session_key = cookie_jar.get_private(SESSION_KEY_REF)?.value().to_string();
        Uuid::parse_str(&session_key).ok()
    }

    pub fn set_session_key(cookie_jar: &CookieJar<'_>, uuid: Uuid) {
        cookie_jar.add_private(Cookie::new(SESSION_KEY_REF, uuid.to_string()))
    }

    pub fn remove_session_key(cookie_jar: &CookieJar<'_>) {
        cookie_jar.remove_private(SESSION_KEY_REF);
    }
}

pub async fn add_session(state: T, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) {
    let id = Uuid::new_v4();
    let session = Session::new(state);
    session_handler.add(id.clone(), session).await;
    set_session_key(cookie_jar, id);
}

pub async fn find_session(cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Option<Session> {
    let session_key = get_session_key(cookie_jar)?;
    session_handler.get(session_key).await
}

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