use crate::log;

pub struct Buffer {
    pub file: Option<String>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn from_file(file: Option<String>) -> Self {
        if let Some(val) = &file {
            println!("the value is {} ", val);
        }

        let lines = match &file {
            Some(val) => std::fs::read_to_string(val)
                .unwrap()
                .lines()
                .map(|s| s.to_string())
                .collect(),
            None => vec![],
        };

        Self { file, lines }
    }
    pub fn get(&self, line: usize) -> Option<String> {
        if self.lines.len() > line {
            return Some(self.lines[line].clone());
        }
        None
    }

    pub fn insert_char(&mut self, x: u16, y: u16, c: char) {
        let line = self.lines.get_mut(y as usize);
        if let Some(line) = line {
            line.insert(x as usize, c);
        }
    }
    pub fn delete_char(&mut self, x: u16, y: u16) {
        let line = self.lines.get_mut(y as usize);
        if let Some(line) = line {
            line.remove(x as usize);
        }
    }

    pub fn delete_line(&mut self, line_no: u16) -> String {
        self.lines.remove(line_no as usize)
    }

    pub fn restore_line(&mut self, line: String, idx: u16) {
        self.lines.insert(idx as usize, line);
    }
    pub fn insert_line(&mut self , idx: u16) {
        self.lines.insert(idx as usize, String::new());
    }
}
