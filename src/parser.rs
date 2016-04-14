use std::io::prelude::*;
use std::fs::File;
use std::str;

pub struct MParser {
    position: usize,
    data: Vec<u8>
}

impl MParser {
    pub fn new(filename: &str) -> Result<MParser, String> {
        let mut file = match File::open(filename) {
            Ok(f)  => f,
            Err(e) => return Err(format!("Couldn't open file: {}", e))
        };

        let mut data = Vec::new();
        match file.read_to_end(&mut data) {
            Ok(_)  => {},
            Err(e) => return Err(format!("Error reading file: {}", e))
        }

        let moov_pos = match find_moov(&data) {
            Some(m) => m,
            None    => return Err("Could not find moov atom in video".to_string())
        };

        let mut chars = &mut [0; 4];
        chars[0] = data[moov_pos];
        chars[1] = data[moov_pos + 1];
        chars[2] = data[moov_pos + 2];
        chars[3] = data[moov_pos + 3];
        println!("Found moov atom at {}, sanity check: {}",
                 moov_pos,
                 str::from_utf8(chars).unwrap());

        Ok(MParser{ position: moov_pos - 4, data: data})
    }

    pub fn move_cursor(&mut self, delta: isize) {
        self.position = ((self.position as isize) + delta) as usize;
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        if self.data.len() < 4 {
            return None;
        }

        let mut r : u32 = 0;

        r = r | (self.data[3] as u32) << 0;
        r = r | (self.data[2] as u32) << 8;
        r = r | (self.data[1] as u32) << 16;
        r = r | (self.data[0] as u32) << 24;

        self.move_cursor(4);

        Some(r)
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        if self.data.len() < 4 {
            return None;
        }

        self.move_cursor(1);

        Some(self.data[0])
    }
}

fn find_moov(data: &[u8]) -> Option<usize> {
    let mut m = 0;
    for i in 0..data.len() {
        match data[i] as char {
            'm' => if m == 0 {
                m = m + 1;
            } else {
                m = 0;
            },
            'o' => if m == 1 || m == 2 {
                m = m + 1;
            } else {
                m = 0;
            },
            'v' => if m == 3 {
                return Some(i-3);
            } else {
                m = 0;
            },
            _ => m = 0
        }
    }

    None
}
