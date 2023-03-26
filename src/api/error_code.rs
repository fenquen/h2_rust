pub type ErrorCode = usize;


pub const NO_DATA_AVAILABLE: ErrorCode = 2000;
pub const URL_FORMAT_ERROR_2: ErrorCode = 90046;

pub fn get_state(error_code: ErrorCode) -> String {
    match error_code {
        NO_DATA_AVAILABLE => "02000".to_string(),
        _ => error_code.to_string(),
    }
}