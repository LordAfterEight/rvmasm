pub struct Memory {
    pub data: Vec<u8>,
    pub labels: Vec<Label>,
    pub blocks: Vec<Block>,
    pub block_tracker: usize,
    pub mem_ptr: usize,
}

pub struct Label {
    pub name: String,
    pub address: usize,
}

pub struct Variable {
    pub name: String,
    pub address: usize,
    pub length: usize,
    pub value: Vec<u8>,
}

pub struct Block {
    pub name: String,
    pub labels: Vec<Label>,
    pub variables: Vec<Variable>,
    pub address: usize,
    pub length: usize,
    pub data: Vec<u8>,
}

impl Label {
    pub fn new(name: &str, address: usize) -> Self {
        Self {
            name: name.to_string(),
            address,
        }
    }
}
