use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use subtle::ConstantTimeEq;

use crate::config::CONFIG;
use crate::config::session::SessionDriver;
use crate::logger;
use axum::response::IntoResponse;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};
use data_encoding::{BASE64URL_NOPAD, HEXLOWER};
use ring::{
    digest,
    rand::{SecureRandom, SystemRandom},
};

const CSRF_KEY: &str = "_csrf";
const FLASH_PREFIX: &str = "_flash.";
const CSRF_LEN: usize = 32;

/// 32 random bytes, base64url encoded. Used for session ids and CSRF tokens.
fn random_token() -> String {
    let mut buf = [0u8; 32];
    SystemRandom::new()
        .fill(&mut buf)
        .expect("system RNG failure");
    BASE64URL_NOPAD.encode(&buf)
}

fn mask_token(raw: &[u8]) -> String {
    let mut pad = vec![0u8; raw.len()];
    SystemRandom::new()
        .fill(&mut pad)
        .expect("system RNG failure");
    let masked: Vec<u8> = raw.iter().zip(&pad).map(|(a, b)| a ^ b).collect();
    pad.extend_from_slice(&masked);
    BASE64URL_NOPAD.encode(&pad)
}

fn unmask_token(given: &str) -> Option<Vec<u8>> {
    let bytes = BASE64URL_NOPAD.decode(given.as_bytes()).ok()?;
    if bytes.len() != CSRF_LEN * 2 {
        return None;
    }
    let (pad, masked) = bytes.split_at(CSRF_LEN);
    Some(pad.iter().zip(masked).map(|(a, b)| a ^ b).collect())
}

/// The store never sees the raw session id, only its SHA-256.
/// a leaked store therefore cannot be used to hijack live sessions.
fn store_key(id: &str) -> String {
    HEXLOWER.encode(digest::digest(&digest::SHA256, id.as_bytes()).as_ref())
}

// ───────────────────────── Session handle ─────────────────────────

#[derive(Clone, Debug)]
struct Inner {
    id: String,
    /// Previous id that must be removed from the store after rotation.
    old_id: Option<String>,
    /// True when the session was loaded from the store (not created now).
    existed: bool,
    data: HashMap<String, String>,
    /// Nothing is persisted (and no cookie is sent) unless this is set.
    dirty: bool,
}

/// Per-request session handle. Cheap to clone; shared through request extensions.
/// The lock is never held across an `.await`, so a std Mutex is fine.
#[derive(Clone, Debug)]
pub struct Session(Arc<Mutex<Inner>>);

impl Session {
    fn new(id: String, data: HashMap<String, String>, existed: bool) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            id,
            old_id: None,
            existed,
            data,
            dirty: false,
        })))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.0.lock().unwrap().data.get(key).cloned()
    }

    pub fn put(&self, key: &str, value: impl Into<String>) {
        let mut s = self.0.lock().unwrap();
        s.data.insert(key.to_string(), value.into());
        s.dirty = true;
    }

    pub fn forget(&self, key: &str) {
        let mut s = self.0.lock().unwrap();
        if s.data.remove(key).is_some() {
            s.dirty = true;
        }
    }

    /// Reads and removes a value.
    pub fn pull(&self, key: &str) -> Option<String> {
        let mut s = self.0.lock().unwrap();
        let v = s.data.remove(key);
        if v.is_some() {
            s.dirty = true;
        }
        v
    }

    /// One-shot message (e.g. validation errors, "saved!" notices).
    pub fn flash(&self, key: &str, value: impl Into<String>) {
        self.put(&format!("{FLASH_PREFIX}{key}"), value);
    }

    pub fn pull_flash(&self, key: &str) -> Option<String> {
        self.pull(&format!("{FLASH_PREFIX}{key}"))
    }

    /// Issues a new session id and keeps the data.
    /// MUST be called after login (prevents session fixation).
    pub fn regenerate(&self) {
        let mut s = self.0.lock().unwrap();
        if s.existed && s.old_id.is_none() {
            s.old_id = Some(s.id.clone());
        }
        s.id = random_token();
        s.dirty = true;
    }

    /// Clears all data and issues a new id. Use it on logout.
    pub fn invalidate(&self) {
        self.0.lock().unwrap().data.clear();
        self.regenerate();
    }

    // ---- CSRF ----

    /// Creates a fresh raw token and stores it in the session.
    fn rotate_raw(&self) -> Vec<u8> {
        let mut raw = vec![0u8; CSRF_LEN];
        SystemRandom::new()
            .fill(&mut raw)
            .expect("system RNG failure");
        self.put(CSRF_KEY, BASE64URL_NOPAD.encode(&raw));
        raw
    }

    fn raw_token(&self) -> Option<Vec<u8>> {
        self.get(CSRF_KEY)
            .and_then(|t| BASE64URL_NOPAD.decode(t.as_bytes()).ok())
            .filter(|b| b.len() == CSRF_LEN)
    }

    /// Returns a MASKED token, different on every call (BREACH mitigation).
    /// The raw token never leaves the session; the page only ever sees
    /// `pad || (token XOR pad)` with a fresh random pad.
    pub fn csrf_token(&self) -> String {
        let raw = self.raw_token().unwrap_or_else(|| self.rotate_raw());
        mask_token(&raw)
    }

    /// Rotates the underlying token (call after login/logout).
    pub fn regenerate_csrf_token(&self) -> String {
        mask_token(&self.rotate_raw())
    }

    /// Unmasks the submitted token and compares it in constant time.
    /// Never creates a token: a session without one always fails.
    pub fn verify_csrf(&self, given: &str) -> bool {
        let (Some(expected), Some(given)) = (self.raw_token(), unmask_token(given)) else {
            return false;
        };

        expected.ct_eq(&given).into()
    }
    fn take_changes(&self) -> Option<(String, Option<String>, HashMap<String, String>)> {
        let s = self.0.lock().unwrap();
        s.dirty
            .then(|| (s.id.clone(), s.old_id.clone(), s.data.clone()))
    }
}

/// Lets handlers take `session: Session` as an argument.
impl<S: Send + Sync> FromRequestParts<S> for Session {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR) // session layer not installed
    }
}

// ───────────────────────── Store ─────────────────────────

type Data = HashMap<String, String>;

/// Explicit enum instead of a trait object: no dyn/async-trait magic.
/// Add a `Database(sqlx::...)` variant here for the persistent driver.
#[derive(Clone)]
pub enum SessionStore {
    Memory(Arc<Mutex<HashMap<String, (Data, Instant)>>>),
}

impl SessionStore {
    async fn load(&self, key: &str) -> Option<Data> {
        match self {
            Self::Memory(m) => {
                let mut m = m.lock().unwrap();
                let alive = m.get(key).map(|(_, exp)| *exp > Instant::now());
                match alive {
                    Some(true) => m.get(key).map(|(d, _)| d.clone()),
                    Some(false) => {
                        m.remove(key);
                        None
                    }
                    None => None,
                }
            }
        }
    }

    async fn save(&self, key: &str, data: Data, ttl: Duration) {
        match self {
            Self::Memory(m) => {
                let mut m = m.lock().unwrap();
                // Opportunistic cleanup of expired sessions.
                if m.len() > 1024 {
                    let now = Instant::now();
                    m.retain(|_, (_, exp)| *exp > now);
                }
                m.insert(key.to_string(), (data, Instant::now() + ttl));
            }
        }
    }

    async fn destroy(&self, key: &str) {
        match self {
            Self::Memory(m) => {
                m.lock().unwrap().remove(key);
            }
        }
    }
}

// ───────────────────────── Manager + middleware ─────────────────────────

#[derive(Clone)]
pub struct SessionManager {
    store: SessionStore,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        let store = match CONFIG.session.driver {
            SessionDriver::Memory => SessionStore::Memory(Arc::default()),
        };
        Self { store }
    }

    fn ttl(&self) -> Duration {
        Duration::from_secs(CONFIG.session.lifetime_minutes * 60)
    }

    fn cookie(&self, id: &str) -> Option<HeaderValue> {
        let cfg = &CONFIG.session; // global config, same convention as the rest of the project
        let mut c = format!(
            "{}={}; Path=/; HttpOnly; SameSite={}; Max-Age={}",
            cfg.cookie_name,
            id,
            cfg.same_site.as_str(),
            cfg.lifetime_minutes * 60
        );
        if cfg.secure {
            c.push_str("; Secure");
        }
        HeaderValue::from_str(&c).ok()
    }
}

fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .find_map(|pair| {
            let (k, v) = pair.trim().split_once('=')?;
            (k == name).then(|| v.to_string())
        })
}

/// Session middleware. Install it with `from_fn_with_state(manager, session::layer)`.
pub async fn layer(State(mgr): State<SessionManager>, mut req: Request, next: Next) -> Response {
    let loaded = match read_cookie(req.headers(), &CONFIG.session.cookie_name) {
        Some(id) => mgr.store.load(&store_key(&id)).await.map(|d| (id, d)),
        None => None,
    };

    // A client-supplied id that is not in the store is ignored, never adopted.
    let session = match loaded {
        Some((id, data)) => Session::new(id, data, true),
        None => Session::new(random_token(), HashMap::new(), false),
    };
    req.extensions_mut().insert(session.clone());

    let mut res = next.run(req).await;

    if let Some((id, old_id, data)) = session.take_changes() {
        if let Some(old) = old_id {
            mgr.store.destroy(&store_key(&old)).await;
        }
        mgr.store.save(&store_key(&id), data, mgr.ttl()).await;
        // in session::layer, replacing the unconditional append
        match mgr.cookie(&id) {
            Some(v) => {
                res.headers_mut().append(header::SET_COOKIE, v);
            }
            None => {
                // Unreachable after startup validation; kept as a safety net.
                logger::error("failed to build session cookie, check SESSION_* config");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    res
}
