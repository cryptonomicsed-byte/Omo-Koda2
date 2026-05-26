pub mod sealed_memory {
    pub fn seal(data: &[u8], _key: &[u8]) -> Vec<u8> {
        data.to_vec() // stub
    }
    pub fn unseal(data: &[u8], _key: &[u8]) -> Result<Vec<u8>, &'static str> {
        Ok(data.to_vec()) // stub
    }
}
