/// Length-prefixed encoding so no two different items can produce the same signed bytes.
pub struct CanonicalWriter {
    bytes: Vec<u8>,
}

impl CanonicalWriter {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn put_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn put_bytes(&mut self, value: &[u8]) {
        self.put_u64(value.len() as u64);
        self.bytes.extend_from_slice(value);
    }

    pub fn put_str(&mut self, value: &str) {
        self.put_bytes(value.as_bytes());
    }

    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}
