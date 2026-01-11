use md5::{Digest, Md5};

/// Generate MD5 hash of content
pub fn md5_hash(content: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate ID from path
pub fn generate_id(path: &str) -> String {
    md5_hash(path)
}

/// Generate edge ID
pub fn generate_edge_id(source: &str, target: &str, edge_type: &str) -> String {
    md5_hash(&format!("{}->{}:{}", source, target, edge_type))
}
