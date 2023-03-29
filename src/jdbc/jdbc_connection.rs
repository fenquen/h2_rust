use std::collections::HashMap;
use crate::properties_type;
use anyhow::Result;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::session_remote;
use crate::engine::session_remote::SessionRemote;
use crate::h2_rust_common::Properties;

const NUM_SERVERS: &str = "numServers";
const PREFIX_SERVER: &str = "server";

#[derive(Default)]
pub struct JdbcConnection {}

impl JdbcConnection {
    pub fn new(url: String,
               info: Properties,
               user: String,
               password: String,
               forbid_creation: bool) -> Result<Self> {
        let mut jdbc_connection = JdbcConnection {};
        jdbc_connection.init(url, info, user, password, forbid_creation)?;
        Ok(jdbc_connection)
    }

    fn init(&mut self,
            url: String,
            info: Properties,
            user: String,
            password: String,
            forbid_creation: bool) -> Result<()> {
        let mut connection_info = ConnectionInfo::new(url, &info, user, password)?;

        if forbid_creation {
            connection_info.set_property("FORBID_CREATION", "TRUE");
        }

        let mut session_remote = SessionRemote::new(connection_info)?;
        session_remote.connect_embedded_or_server(false);
        Ok(())
    }
}