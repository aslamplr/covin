use serde::Serialize;

use crate::covin::centers::{Center, Session};

#[derive(Debug, Serialize)]
pub struct AlertSession<'a, 'b> {
    pub session: &'a Session,
    pub center: &'b Center,
}

impl<'a, 'b> From<(&'a Session, &'b Center)> for AlertSession<'a, 'b> {
    fn from((session, center): (&'a Session, &'b Center)) -> Self {
        Self { session, center }
    }
}

#[cfg(test)]
impl std::fmt::Display for AlertSession<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let session = self.session;
        write!(f, "{}", session.session_id)
    }
}
