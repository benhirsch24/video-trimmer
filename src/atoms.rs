use parser::MParser;

pub trait AtomParser<T> {
    fn parse(parser: &mut MParser) -> Result<T, String>;
}

pub struct MoovAtom {
    pub size: u32
}

impl MoovAtom {
    fn new(size: u32) -> MoovAtom {
        MoovAtom {
            size: size
        }
    }
}

impl AtomParser<MoovAtom> for MoovAtom {
    fn parse(parser: &mut MParser) -> Result<MoovAtom, String> {
        let size = match parser.read_u32() {
            Some(s) => s,
            None    => return Err("Couldn't read moov size".to_string())
        };

        return Ok(MoovAtom::new(size));
    }
}
