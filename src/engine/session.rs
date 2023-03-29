use crate::engine::session_local::SessionLocal;
use crate::engine::session_remote::SessionRemote;

pub enum Session {
    LocalSession(SessionLocal),
    RemoteSession(SessionRemote),
}