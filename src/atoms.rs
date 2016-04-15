use parser::*;

pub trait AtomParser<T> {
    fn parse(parser: &mut MParser) -> Result<T, String>;
}

pub struct MovieHeaderAtom {
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub time_scale: u32,
    pub duration: u32,
    pub rate: f32,
    pub volume: f32,
    pub matrix: Vec<f32>,
    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32
}

impl MovieHeaderAtom {
    fn default() -> MovieHeaderAtom {
        MovieHeaderAtom {
            size: 0,
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            time_scale: 0,
            duration: 0,
            rate: 0.0f32,
            volume: 0.0f32,
            matrix: Vec::new(),
            preview_time: 0,
            preview_duration: 0,
            poster_time: 0,
            selection_time: 0,
            selection_duration: 0,
            current_time: 0,
            next_track_id:0
        }
    }
}

macro_rules! atom_type {
    ( $e:expr, $parser:ident ) => {
        {
            let typ = try!(TypeParserAction::try_parse($parser));
            if typ != $e.to_string() {
                return Err(format!("Type of atom didn't match; expected \"mvhd\" got {}", typ));
            }
            typ
        }
    }
}

impl AtomParser<MovieHeaderAtom> for MovieHeaderAtom {
    fn parse(parser: &mut MParser) -> Result<MovieHeaderAtom, String> {
        let mut movie_header = MovieHeaderAtom::default();

        let size = try!(U32ParserAction::try_parse(parser));
        let typ = atom_type!("mvhd", parser);
        let version = try!(U32ParserAction::try_parse(parser));

        unimplemented!();
    }
}

pub struct MoovAtom {
    pub size: u32,
    pub mvhd: Option<MovieHeaderAtom>
}

impl MoovAtom {
    fn new(size: u32, mvhd: Option<MovieHeaderAtom>) -> MoovAtom {
        MoovAtom {
            size: size,
            mvhd: mvhd
        }
    }
}

macro_rules! parse_atom {
    ( $atom:ident, $parser:ident ) => {
        {
            $parser.push_stack();
            try!(
                match $atom::parse($parser) {
                Ok(a)  => Ok(a),
                Err(e) => {
                    $parser.unwind();
                    Err(e)
                }
                }
            );
        }
    }
}

impl AtomParser<MoovAtom> for MoovAtom {
    fn parse(parser: &mut MParser) -> Result<MoovAtom, String> {
        let size = match parser.read_u32() {
            Some(s) => s,
            None    => return Err("Couldn't read moov size".to_string())
        };

        let mvhd_atom = parse_atom!(MovieHeaderAtom, parser);

        return Ok(MoovAtom::new(size, None));
    }
}
