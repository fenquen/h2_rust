use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct DbError {
    /// 有errCode转化而来
    pub sql_state: String,

    /// 通过sql_state对照账单得到相应的话术
    pub message: String,

    pub error_code: u64,

}

impl DbError {
    pub fn get(error_code:u64){

    }

    pub fn get_state(){

    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "message:{},sql_state:{},error_code:{}",
               self.message, self.sql_state, self.error_code)
    }
}

impl Error for DbError {}

