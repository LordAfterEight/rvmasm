pub fn blockify(source: String) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
    let mut blocks: Vec<Block> = Vec::new();

    let mut lines: Vec<Vec<String>> = source
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().split_whitespace().map(str::to_string).collect())
        .collect();

    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        if line[0] == "@BLCK" {
            let mut block = Block::new();
            block.name = line[1].clone();
            block.base = Some(parse_single_value(&line[2])?);
            i += 1;
            while i < lines.len() && lines[i][0] != "}" {                                                                                                                                                     
                block.content.push(lines[i].clone());
                i += 1;                                                                                                                                                                                       
            }   
            blocks.push(block);
        }
        i += 1;
    }
    Ok(blocks)
}

pub struct Block {
    pub name: String,
    pub base: Option<u32>,
    pub content: Vec<Vec<String>>,
}

impl Block {
    pub fn new() -> Self {
        Self {
            name: "Unnamed".to_string(),
            base: None,
            content: Vec::new(),
        }
    }
}

pub fn parse_single_value(val: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(0)
}
