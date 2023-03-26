use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::properties_type;
use anyhow::Result;
use crate::common::{constant as common_constant, Properties};

pub fn load_properties(prop_file_path: &Path) -> Result<Properties> {
    let prop_file = File::open(prop_file_path)?;
    let buf_reader = BufReader::new(prop_file);

    let mut map = HashMap::<String, String>::new();

    for line_result in buf_reader.lines() {
        let line = line_result?;
        let element_vec: Vec<_> = line.split(common_constant::EQUAL).collect();
        map.insert(element_vec[0].to_string(), element_vec[1].to_string());
    }

    Ok(map)
}