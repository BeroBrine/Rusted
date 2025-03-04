use crate::editor::main_editor::InsertModeTextAddInfo;

pub struct Buffer {
    pub file: Option<String>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new(file: Option<String>, content: Option<String>) -> Self {
        let lines = match &file {
            Some(_) => content.unwrap().lines().map(|s| s.to_string()).collect(),
            None => vec![],
        };
        Self { file, lines }
    }

    pub fn from_file(file: Option<String>) -> Self {
        let buf = match &file {
            Some(str) => Self::new(
                Some(str.to_string()),
                Some(std::fs::read_to_string(str).unwrap()),
            ),
            None => Self::new(file, None),
        };

        buf
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
    pub fn insert_line(&mut self, idx: u16) {
        self.lines.insert(idx as usize, String::new());
    }

    pub fn remove_insert_changes(&mut self, insert_changes: InsertModeTextAddInfo) {
        let indexes = insert_changes.index;
        let starting_index = indexes.0 as usize;
        let ending_index = indexes.1 as usize;
        let line_no = insert_changes.line_no;

        let mut string = self.lines.remove(line_no as usize);
        string.replace_range(starting_index..=ending_index, "");
        if !string.trim().is_empty() {
            self.lines.insert(line_no as usize, string);
        }
    }

    pub fn viewport_buf(&self, vtop: usize, vheight: usize) -> String {
        let height = std::cmp::min(vtop + vheight, self.lines.len());
        self.lines[vtop..height].join("\n")
    }
}
