use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Add, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use anyhow::Result;
use lazy_static::lazy_static;
use crate::api::error_code;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::constant;
use crate::engine::database::{Database, DatabaseRef};
use crate::engine::session_local::SessionLocal;
use crate::h2_rust_common::{h2_rust_constant, Nullable};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::message::db_error::DbError;
use crate::store::fs::file_utils;
use crate::{get_ref_mut, throw};

lazy_static! {
    static ref DATABASE_PATH_DATABASE_HOLDER:Mutex<HashMap<String,Arc<H2RustCell<DatabaseHolder>>>> = Mutex::new(HashMap::new());
}

pub fn create_session(connection_info: &mut ConnectionInfo) -> Result<SessionLocal> {
    open_session(connection_info);
    todo!()
}

fn open_session(connection_info: &mut ConnectionInfo) -> Result<SessionLocal> {
    let if_exist = connection_info.remove_property_bool("IFEXIST", false)?;
    let forbid_creation = connection_info.remove_property_bool("FORBID_CREATION", false)?;
    let ignore_unknown_setting = connection_info.remove_property_bool("IGNORE_UNKNOWN_SETTINGS", false)?;
    let cipher = connection_info.remove_property_str("CIPHER", h2_rust_constant::EMPTY_STR);
    let init = connection_info.remove_property_str("INIT", h2_rust_constant::EMPTY_STR);

    let start = Instant::now();
    open_session1(connection_info, if_exist, forbid_creation, &cipher);
    todo!()
}

fn open_session1(connection_info: &mut ConnectionInfo,
                 if_exist: bool,
                 force_creation: bool,
                 cipher: &String) -> Result<SessionLocal> {
    connection_info.remove_property_bool("NO_UPGRADE", false);

    let open_new = connection_info.get_property_bool("OPEN_NEW", false)?;
    let mut opened = false;

    let database_path = connection_info.get_database_path()?;

    let database_holder = if connection_info.unnamed_in_memory {
        Arc::new(H2RustCell::new(DatabaseHolder::new()))
    } else {
        let mut mutex_guard = DATABASE_PATH_DATABASE_HOLDER.lock().unwrap();
        // let mut r = mutex_guard.borrow_mut();

        if !mutex_guard.contains_key(&database_path) {
            let database_holder = Arc::new(H2RustCell::new(DatabaseHolder::new()));
            mutex_guard.insert(database_path.to_string(), database_holder.clone()); //
            database_holder
        } else {
            mutex_guard.get(&database_path).unwrap().clone()
        }
    };

    {
        let database_holder = database_holder.get_ref_mut();

        let mutex_guard = database_holder.mutex.lock().unwrap();
        if database_holder.database.is_none() || open_new {
            if connection_info.persistent {
                let value = connection_info.get_property("MV_STORE");
                let mut file_name = database_path.clone().add(constant::SUFFIX_MV_FILE);

                let file_name: Result<Option<String>> = if value.is_none() {
                    if !file_utils::exist(&file_name) {
                        throw_not_found(if_exist, force_creation, &database_path)?;

                        file_name = file_name.add(constant::SUFFIX_OLD_DATABASE_FILE);

                        if file_utils::exist(&file_name) {
                            throw!(DbError::get(error_code::FILE_VERSION_ERROR_1,
                                             vec![&format!("old database: {} - please convert the database to a SQL script and re-create it.", file_name)]));
                        }

                        Ok(None)
                    } else {
                        Ok(Some(file_name))
                    }
                } else {
                    if !file_utils::exist(&file_name) {
                        throw_not_found(if_exist, force_creation, &database_path)?;
                        Ok(None)
                    } else {
                        Ok(Some(file_name))
                    }
                };

                let file_name = file_name?;

                if file_name.is_some() && !file_utils::can_write(file_name.unwrap()) {
                    connection_info.set_property("ACCESS_MODE_DATA", "r");
                }
            } else {
                throw_not_found(if_exist, force_creation, &database_path)?;
            }

            let database = Database::new(connection_info, cipher)?; // 参数connectionInfo只是为了提供信息不是委身
            opened = true;
            let found = false;


            database_holder.database = database;
        }
    }

    todo!()
}

fn throw_not_found(if_exist: bool, forbid_creation: bool, name: &str) -> Result<()> {
    if if_exist {
        throw!(DbError::get(error_code::DATABASE_NOT_FOUND_WITH_IF_EXISTS_1, vec![name]));
    } else if forbid_creation {
        throw!(DbError::get(error_code::REMOTE_DATABASE_NOT_FOUND_1, vec![name]));
    }

    Ok(())
}

#[derive(Default)]
struct DatabaseHolder {
    mutex: Mutex<()>,
    database: DatabaseRef,
}

impl DatabaseHolder {
    pub fn new() -> DatabaseHolder {
        DatabaseHolder::default()
    }
}

mod test {
    use std::ops::Deref;
    use std::sync::Arc;
    use crate::h2_rust_common::Integer;

    #[test]
    fn test_arc() {
        let value = Arc::new(1);
        value.clone();
        let b: Box<Integer> = Box::new(1);
    }
}