//! Session-only file identity cache. The first use still verifies all bytes.
#[derive(PartialEq, Eq)]
pub struct Stamp {
    length: u64,
    modified: std::time::SystemTime,
    hash: String,
    #[cfg(unix)]
    identity: (u64, u64, i64, i64),
}
impl Stamp {
    pub fn new(meta: &std::fs::Metadata, hash: &str) -> Result<Self, String> {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Ok(Self {
            length: meta.len(),
            modified: meta.modified().map_err(super::err)?,
            hash: hash.into(),
            #[cfg(unix)]
            identity: (meta.dev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
        })
    }
}
