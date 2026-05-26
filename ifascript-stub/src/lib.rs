pub mod vm {
    pub fn eval(_input: &str) -> String { String::new() }
}
pub mod odu {
    pub const ODU_TABLE: &[(&str, u8)] = &[];
    pub fn lookup(_name: &str) -> Option<u8> { None }
}
pub mod entropy {
    pub fn generate(_seed: &[u8]) -> Vec<u8> { vec![] }
}
pub mod ebo {
    pub fn cast(_odu: u8) -> &'static str { "" }
}
