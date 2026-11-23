//! Authenticated session and the store shared with the HTTP middleware.

use std::fmt;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use polito_api::models::Identity;

/// Identity returned by login / switch career.
///
/// Wraps the generated [`Identity`] without exposing it: a change in the spec
/// stays behind these accessors instead of breaking the SDK public API.
#[derive(Clone, PartialEq)]
pub struct Session {
    identity: Identity,
}

impl Session {
    pub(crate) fn from_identity(identity: Identity) -> Self {
        Self { identity }
    }

    // TODO: `Session::restore`, to resume a session saved by the caller
    // without logging in again.

    pub fn username(&self) -> &str {
        &self.identity.username
    }

    /// Identity type as reported by the API (e.g. student).
    pub fn kind(&self) -> &str {
        &self.identity.r#type
    }

    pub fn client_id(&self) -> &str {
        &self.identity.client_id
    }

    pub fn token(&self) -> &str {
        &self.identity.token
    }
}

// the token never ends up in logs
impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("username", &self.username())
            .field("kind", &self.kind())
            .field("client_id", &self.client_id())
            .field("token", &"<redacted>")
            .finish()
    }
}

/// Current session, shared between the client and [`crate::middleware::AuthMiddleware`].
///
/// A poisoned lock is recovered: the data is a plain `Option<Session>` that a
/// panicking writer cannot leave half-updated.
#[derive(Clone, Default, Debug)]
pub(crate) struct SessionStore(Arc<RwLock<Option<Session>>>);

impl SessionStore {
    pub(crate) fn get(&self) -> Option<Session> {
        self.read().clone()
    }

    pub(crate) fn token(&self) -> Option<String> {
        self.read().as_ref().map(|s| s.token().to_owned())
    }

    pub(crate) fn set(&self, session: Session) {
        *self.write() = Some(session);
    }

    pub(crate) fn clear(&self) -> Option<Session> {
        self.write().take()
    }

    fn read(&self) -> RwLockReadGuard<'_, Option<Session>> {
        self.0.read().unwrap_or_else(|e| e.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, Option<Session>> {
        self.0.write().unwrap_or_else(|e| e.into_inner())
    }
}
