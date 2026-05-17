use uuid::Uuid;

pub fn generate_stored_name(extension: &str) -> String {
    let ext = extension.trim_start_matches('.');
    format!("{}.{}", Uuid::new_v4(), ext)
}
