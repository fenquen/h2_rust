pub fn name_separators_to_native(path: &str) -> String {
    path.replace("\\", "/")
}