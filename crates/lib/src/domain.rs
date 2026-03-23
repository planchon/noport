pub fn generate_domain(path: &String) -> Option<String> {
    // get the name of the folder
    if let Some(val) = path.split("/").last() {
        return Some(val.to_string());
    }

    None
}
