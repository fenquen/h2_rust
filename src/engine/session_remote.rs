use anyhow::Result;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::engine;
use crate::engine::session::Session;
use crate::h2_rust_common::Integer;

const SESSION_PREPARE: Integer = 0;
const SESSION_CLOSE: Integer = 1;
const COMMAND_EXECUTE_QUERY: Integer = 2;
const COMMAND_EXECUTE_UPDATE: Integer = 3;
const COMMAND_CLOSE: Integer = 4;
const RESULT_FETCH_ROWS: Integer = 5;
const RESULT_RESET: Integer = 6;
const RESULT_CLOSE: Integer = 7;
const COMMAND_COMMIT: Integer = 8;
const CHANGE_ID: Integer = 9;
const COMMAND_GET_META_DATA: Integer = 10;

// 11 was used for SESSION_PREPARE_READ_PARAMS

const SESSION_SET_ID: Integer = 12;
const SESSION_CANCEL_STATEMENT: Integer = 13;
const SESSION_CHECK_KEY: Integer = 14;
const SESSION_SET_AUTOCOMMIT: Integer = 15;
const SESSION_HAS_PENDING_TRANSACTION: Integer = 16;
const LOB_READ: Integer = 17;
const SESSION_PREPARE_READ_PARAMS2: Integer = 18;
const GET_JDBC_META: Integer = 19;

const STATUS_ERROR: Integer = 0;
const STATUS_OK: Integer = 1;
const STATUS_CLOSED: Integer = 2;
const STATUS_OK_STATE_CHANGED: Integer = 3;

pub struct SessionRemote {
    connection_info: ConnectionInfo,
    old_information_schema: bool,
}

impl SessionRemote {
    pub fn new(connection_info: ConnectionInfo) -> Result<Self> {
        let o = connection_info.get_property_bool("OLD_INFORMATION_SCHEMA", false)?;

        Ok(SessionRemote {
            connection_info,
            old_information_schema: o,
        })
    }

    pub fn connect_embedded_or_server(&mut self, open_new: bool) -> Result<Session> {
        let auto_server_mode = self.connection_info.get_property_bool("AUTO_SERVER", false)?;
        if auto_server_mode {
            todo!()
        }

        if open_new {
            self.connection_info.set_property("OPEN_NEW", "true");
        }

        engine::create_session(&mut self.connection_info);
        todo!()
    }
}
