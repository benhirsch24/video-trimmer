use std::io::prelude::*;
use std::fs::File;

macro_rules! check_length {
    ( $x:expr, $p:ident, $t:expr ) => {
        if $p.get_remaining_bytes() < $x {
            return Err(format!("{} not enough bytes: need {} have {}", $t, $x, $p.get_remaining_bytes()));
        }
    };
}

pub struct MParser {
    position: usize,
    pub data: Vec<u8>,
    size: usize
}

impl MParser {
    pub fn get_position(&self) -> usize { self.position }

    pub fn set_position(&mut self, new_position: usize) { self.position = new_position; }

    pub fn get_size(&self) -> usize { self.size }

    pub fn get_remaining_bytes(&self) -> usize { self.data.len() - self.position }

    pub fn get_byte(&self, offset: usize) -> u8 {
        self.data[self.position + offset]
    }

    pub fn new(filename: &str) -> Result<MParser, String> {
        let mut file = match File::open(filename) {
            Ok(f)  => f,
            Err(e) => return Err(format!("Couldn't open video in: {}", e))
        };

        let mut data = Vec::new();
        match file.read_to_end(&mut data) {
            Ok(_)  => {},
            Err(e) => return Err(format!("Couldn't read data in video: {}", e))
        };

        let size = data.len();

        Ok(MParser{ position: 0, data: data, size: size })
    }

    pub fn move_cursor(&mut self, delta: isize) -> Result<(), String> {
        let new_position = ((self.position as isize) + delta) as usize;

        if new_position > self.size {
            return Err(format!("New parser position {} > parser size {}", new_position, self.size));
        }

        self.position = new_position;
        Ok(())
    }

    pub fn read_u32(&mut self) -> Result<u32, String> {
        check_length!(4, self, "read_u32");

        let r = {
            let data = &self.data[self.position .. self.position+4];

            let mut r : u32 = 0;

            r = r | ((data[3] as u32) << 0);
            r = r | ((data[2] as u32) << 8);
            r = r | ((data[1] as u32) << 16);
            r = r | ((data[0] as u32) << 24);

            r
        };

        try!(self.move_cursor(4));

        Ok(r)
    }

    pub fn read_u16(&mut self) -> Result<u16, String> {
        check_length!(2, self, "read_u16");

        let r = {
            let data = &self.data[self.position .. self.position+2];

            let mut r : u16 = 0;

            r = r | ((data[1] as u16) << 0);
            r = r | ((data[0] as u16) << 8);

            r
        };

        try!(self.move_cursor(2));

        Ok(r)
    }

    pub fn read_flags(&mut self) -> Result<u32, String> {
        check_length!(3, self, "read_flags");

        let r = {
            let data = &self.data[self.position .. self.position+3];

            let mut r : u32 = 0;

            r = r | ((data[2] as u32) << 0);
            r = r | ((data[1] as u32) << 8);
            r = r | ((data[0] as u32) << 16);

            r
        };

        try!(self.move_cursor(3));

        Ok(r)
    }

    pub fn read_u8(&mut self) -> Result<u8, String> {
        check_length!(1, self, "read_u8");

        try!(self.move_cursor(1));

        Ok(self.data[self.position])
    }

    // read_u32 moves cursor
    pub fn read_fixed32(&mut self) -> Result<f32, String> {
        let integer = try!(self.read_u32());
        let float = (integer as f32) / ((!0 as u32) as f32);

        Ok(float)
    }

    // read_u16 moves cursor
    pub fn read_fixed16(&mut self) -> Result<f32, String> {
        let integer = try!(self.read_u16());
        let float = (integer as f32) / ((!0 as u16) as f32);

        Ok(float)
    }

    pub fn get_view_at(&mut self, position: usize) -> MParserView {
        MParserView::new(position, self)
    }
}

pub struct MParserView<'a> {
    initial_position: usize,
    parser: &'a mut MParser
}

impl<'a> MParserView<'a> {
    fn new(position: usize, parser: &mut MParser) -> MParserView {
        parser.set_position(position);
        MParserView {
            initial_position: position,
            parser: parser
        }
    }
}

impl<'a> Drop for MParserView<'a> {
    fn drop(&mut self) {
        self.parser.set_position(self.initial_position);
    }
}

impl<'a> MParserView<'a> {
    pub fn clone_view(&mut self) -> MParserView { MParserView::new(self.initial_position, self.parser) }

    pub fn move_cursor(&mut self, delta: isize) -> Result<(), String> { self.parser.move_cursor(delta) }
    pub fn get_position(&self) -> usize { self.parser.get_position() }
    pub fn set_position(&mut self, new_position: usize) { self.parser.set_position(new_position); }

    pub fn get_byte(&self, offset: usize) -> u8 { self.parser.get_byte(offset) }
    pub fn get_remaining_bytes(&self) -> usize { self.parser.get_remaining_bytes() }

    pub fn read_u32(&mut self) -> Result<u32, String> { self.parser.read_u32() }
    pub fn read_u16(&mut self) -> Result<u16, String> { self.parser.read_u16() }
    pub fn read_u8(&mut self)  -> Result<u8, String>  { self.parser.read_u8() }
    pub fn read_fixed32(&mut self) -> Result<f32, String> { self.parser.read_fixed32() }
    pub fn read_fixed16(&mut self) -> Result<f32, String> { self.parser.read_fixed16() }
    pub fn read_flags(&mut self) -> Result<u32, String> { self.parser.read_flags() }
}

pub trait ParserAction<T> {
    fn try_parse(parser: &mut MParserView) -> Result<T, String>;
}

pub struct TypeParserAction;
impl ParserAction<String> for TypeParserAction {
    fn try_parse(parser: &mut MParserView) -> Result<String, String> {
        check_length!(4, parser, "TypeParserAction");

        let mut bytes = Vec::new();
        bytes.push(parser.get_byte(0));
        bytes.push(parser.get_byte(1));
        bytes.push(parser.get_byte(2));
        bytes.push(parser.get_byte(3));

        match String::from_utf8(bytes) {
            Ok(s)  => {
                try!(parser.move_cursor(4));

                Ok(s)
            },
            Err(e) => Err(format!("Error parsing type string: {}", e))
        }
    }
}
