use parser::*;
use std::fmt;

/* ============================ Traits and helpers ============================ */

pub fn atom_type_and_size(parser: &mut MParserView) -> Result<(u32, String), String> {
    let size = try!(parser.read_u32());
    let typ  = try!(TypeParserAction::try_parse(parser));

    Ok((size, typ))
}

fn loop_and_get_children(parser: &mut MParserView, atoms: &[&str]) -> Result<Vec<usize>, String> {
    let mut atom_positions = vec![];

    //try!(parser.move_cursor(8));
    loop {
        let (size, typ) = match atom_type_and_size(parser) {
            Ok((s,t)) => (s, t),
            Err(_)    => break
        };

        let actual_pos = parser.get_position() - 8;
        if atoms.contains(&typ.as_str()) {
            atom_positions.push(actual_pos);
            parser.set_position(actual_pos + (size as usize));
        } else {
            break;
        }
    }

    atom_positions.reverse();
    Ok(atom_positions)
}

pub trait AtomParser {
    fn parse(&mut self, parser: &mut MParserView, depth: usize) -> Result<(), String> {
        let atom_position = parser.get_position();
        try!(self.parse_self(parser));

        // get children and add them to the stack
        let mut atom_position_stack = vec![];
        {
            let mut view = parser.get_view_at(atom_position);
            let mut children = try!(self.get_children(&mut view));
            atom_position_stack.append(&mut children);
        }

        while atom_position_stack.len() > 0
        {
            // pop next atom position off stack which should exist
            let stack_pos = atom_position_stack.pop().unwrap();

            // visit (parse the atom)
            {
                let mut view = parser.get_view_at(stack_pos);

                let (size, typ) = match atom_type_and_size(&mut view) {
                    Ok((s,t)) => (s, t),
                    Err(_)    => break
                };

                for _ in 0..(depth*3) {
                    print!(" ");
                }
                println!("{} @ {} with size {} (stack len = {})", typ, stack_pos, size, atom_position_stack.len());

                try!(self.parse_child(&typ, &mut view, depth + 1));
            }
        };

        Ok(())
    }

    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String>;

    fn get_children(&self, _: &mut MParserView) -> Result<Vec<usize>, String>
    {
        Ok(vec![])
    }

    fn parse_child(&mut self, _: &str, _: &mut MParserView, _: usize) -> Result<(), String> {
        Ok(())
    }
}

/* ================================= Actual atoms ================================= */

pub struct MovieAtoms {
    pub moov: Option<MoovAtom>
}

impl MovieAtoms {
    pub fn new() -> MovieAtoms {
        MovieAtoms {
            moov: None
        }
    }
}

impl AtomParser for MovieAtoms {
    fn parse_self(&mut self, _: &mut MParserView) -> Result<(), String> {
        Ok(())
    }

    fn parse_child(&mut self, atom: &str, parser: &mut MParserView, depth: usize) -> Result<(), String> {
        match atom {
            "moov" => {
                let mut moov = MoovAtom::new();
                try!(moov.parse(parser, depth));
                self.moov = Some(moov);
            },
            _      => { println!("Need to parse {}", atom); }
        };

        Ok(())
    }

    fn get_children(&self, parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec!["moov"];
        let children = try!(loop_and_get_children(parser, &atoms));

        Ok(children)
    }
}

pub struct MoovAtom {
    pub location: usize,
    pub size: u32,
    pub mvhd: Option<MovieHeaderAtom>,
    pub traks: Vec<TrakAtom>
}

impl MoovAtom {
    pub fn new() -> MoovAtom {
        MoovAtom {
            location: 0,
            size: 0,
            mvhd: None,
            traks: vec![]
        }
    }
}

impl AtomParser for MoovAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location = parser.get_position();
        self.size     = try!(parser.read_u32());

        try!(parser.move_cursor(4));

        Ok(())
    }

    fn parse_child(&mut self, atom: &str, parser: &mut MParserView, depth: usize) -> Result<(), String> {
        match atom {
            "mvhd" => {
                let mut mvhd = MovieHeaderAtom::new();
                try!(mvhd.parse(parser, depth));
                self.mvhd = Some(mvhd);
            },
            "trak" => {
                let mut trak = TrakAtom::new();
                try!(trak.parse(parser, depth));
                self.traks.push(trak);
            },
            _      => { println!("Need to parse {}", atom); }
        };
        Ok(())
    }

    fn get_children(&self, parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec!["mvhd", "iods", "trak", "udta"];
        let children = try!(loop_and_get_children(parser, &atoms));

        Ok(children)
    }
}

impl fmt::Display for MoovAtom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Atom moov @ {} of size {}; ends at {}",
               self.location,
               self.size,
               (self.location as u32) + self.size)
    }
}

pub struct MovieHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub time_scale: u32,
    pub duration: u32,
    pub rate: f32,
    pub volume: f32,
    pub preview_time: u32,
    pub preview_duration: u32,
    pub poster_time: u32,
    pub selection_time: u32,
    pub selection_duration: u32,
    pub current_time: u32,
    pub next_track_id: u32
}

impl MovieHeaderAtom {
    fn new() -> MovieHeaderAtom
    {
        MovieHeaderAtom {
            location: 0,
            size: 0,
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            time_scale: 0,
            duration: 0,
            rate: 0.0f32,
            volume: 0.0f32,
            preview_time: 0,
            preview_duration: 0,
            poster_time: 0,
            selection_time: 0,
            selection_duration: 0,
            current_time: 0,
            next_track_id: 0
        }
    }
}

impl AtomParser for MovieHeaderAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location                   = parser.get_position();
        self.size                       = try!(parser.read_u32());
        try!(parser.move_cursor(4)); // reserved
        self.version                    = try!(parser.read_u8());
        self.flags                      = try!(parser.read_flags());
        self.creation_time              = try!(parser.read_u32());
        self.modification_time          = try!(parser.read_u32());
        self.time_scale                 = try!(parser.read_u32());
        self.duration                   = try!(parser.read_u32());
        self.rate                       = try!(parser.read_fixed32());
        self.volume                     = try!(parser.read_fixed16());
        try!(parser.move_cursor(36)); // matrix
        self.preview_time               = try!(parser.read_u32());
        self.preview_duration           = try!(parser.read_u32());
        self.poster_time                = try!(parser.read_u32());
        self.selection_time             = try!(parser.read_u32());
        self.selection_duration         = try!(parser.read_u32());
        self.current_time               = try!(parser.read_u32());
        self.next_track_id              = try!(parser.read_u32());

        Ok(())
    }
}

pub struct TrakAtom {
    pub location: usize,
    pub size: u32,
    pub tkhd: Option<TrakHeaderAtom>,
    pub mdia: Option<MediaAtom>
}

impl TrakAtom {
    fn new() -> TrakAtom {
        TrakAtom {
            location: 0,
            size: 0,
            tkhd: None,
            mdia: None
        }
    }
}

impl AtomParser for TrakAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location = parser.get_position();
        self.size     = try!(parser.read_u32());
        Ok(())
    }

    fn parse_child(&mut self, atom: &str, parser: &mut MParserView, depth: usize) -> Result<(), String> {
        match atom {
            "tkhd" => {
                let mut tkhd = TrakHeaderAtom::new();
                try!(tkhd.parse(parser, depth));
                self.tkhd = Some(tkhd);
            },
            "mdia" => {
                let mut mdia = MediaAtom::new();
                try!(mdia.parse(parser, depth));
                self.mdia = Some(mdia);
            },
            _      => { println!("Need to parse {}", atom); }
        };

        Ok(())
    }

    fn get_children(&self, parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec![
            "tkhd", "tapt", "clip", "matt", "edts", "tref",
            "txas", "load", "imap", "mdia", "udta"
        ];
        let children = try!(loop_and_get_children(parser, &atoms));

        Ok(children)
    }
}

pub struct TrakHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub track_id: u32,
    pub duration: u32,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: f32,
    pub track_width: f32,
    pub track_height: f32
}

impl TrakHeaderAtom {
    fn new() -> TrakHeaderAtom {
        TrakHeaderAtom {
            location: 0,
            size: 0,
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: 0.0f32,
            track_width: 0.0f32,
            track_height: 0.0f32
        }
    }
}

impl AtomParser for TrakHeaderAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location                  = parser.get_position();
        self.size                      = try!(parser.read_u32());
        try!(parser.move_cursor(4));
        self.version                   = try!(parser.read_u8());
        self.flags                     = try!(parser.read_flags());
        self.creation_time             = try!(parser.read_u32());
        self.modification_time         = try!(parser.read_u32());
        self.track_id                  = try!(parser.read_u32());
        try!(parser.move_cursor(4));  // reserved
        self.duration                  = try!(parser.read_u32());
        try!(parser.move_cursor(8));  // reserved
        self.layer                     = try!(parser.read_u16());
        self.alternate_group           = try!(parser.read_u16());
        self.volume                    = try!(parser.read_fixed16());
        try!(parser.move_cursor(2));  // reserved
        try!(parser.move_cursor(36)); // matrix
        self.track_width               = try!(parser.read_fixed32());
        self.track_height              = try!(parser.read_fixed32());

        Ok(())
    }
}

pub struct MediaAtom {
    pub location: usize,
    pub size: u32,
    pub mdhd: Option<MediaHeaderAtom>,
    pub hdlr: Option<HandlerReferenceAtom>,
    //pub minf: Option<MediaInfoAtom>,
    //pub udta: Option<UserDataAtom>
}

impl MediaAtom {
    fn new() -> MediaAtom {
        MediaAtom {
            location: 0,
            size: 0,
            mdhd: None,
            hdlr: None
        }
    }
}

impl AtomParser for MediaAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location                 = parser.get_position();
        self.size                     = try!(parser.read_u32());
        try!(parser.move_cursor(4));

        Ok(())
    }

    fn parse_child(&mut self, atom: &str, parser: &mut MParserView, depth: usize) -> Result<(), String> {
        match atom {
            "mdhd" => {
                let mut mdhd = MediaHeaderAtom::new();
                try!(mdhd.parse(parser, depth));
                self.mdhd = Some(mdhd);
            },
            "hdlr" => {
                let mut hdlr = HandlerReferenceAtom::new();
                try!(hdlr.parse(parser, depth));
                self.hdlr = Some(hdlr);
            },
            _      => { println!("Need to parse {}", atom); }
        };

        Ok(())
    }

    fn get_children(&self, parser: &mut MParserView) -> Result<Vec<usize>, String> {
        let atoms = vec!["mdhd", "elng", "hdlr", "minf", "udta"];
        let children = try!(loop_and_get_children(parser, &atoms));

        Ok(children)
    }
}

pub struct MediaHeaderAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub creation_time: u32,
    pub modification_time: u32,
    pub time_scale: u32,
    pub duration: u32,
    pub language: u16,
    pub quality: u16
}

impl MediaHeaderAtom {
    fn new() -> MediaHeaderAtom
    {
        MediaHeaderAtom {
            location: 0, size: 0, version: 0, flags: 0,
            creation_time: 0, modification_time: 0,
            time_scale: 0, duration: 0, language: 0, quality: 0
        }
    }
}

impl AtomParser for MediaHeaderAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location                 = parser.get_position();
        self.size                     = try!(parser.read_u32());
        try!(parser.move_cursor(4));
        self.version                  = try!(parser.read_u8());
        self.flags                    = try!(parser.read_flags());
        self.creation_time            = try!(parser.read_u32());
        self.modification_time        = try!(parser.read_u32());
        self.time_scale               = try!(parser.read_u32());
        self.duration                 = try!(parser.read_u32());
        self.language                 = try!(parser.read_u16());
        self.quality                  = try!(parser.read_u16());

        Ok(())
    }
}

pub struct HandlerReferenceAtom {
    pub location: usize,
    pub size: u32,
    pub version: u8,
    pub flags: u32,
    pub component_type: u32,
    pub component_subtype: u32,
}

impl HandlerReferenceAtom {
    fn new() -> HandlerReferenceAtom
    {
        HandlerReferenceAtom {
            location: 0, size: 0, version: 0, flags: 0,
            component_type: 0, component_subtype: 0
        }
    }
}

impl AtomParser for HandlerReferenceAtom {
    fn parse_self(&mut self, parser: &mut MParserView) -> Result<(), String> {
        self.location = parser.get_position();
        self.size = try!(parser.read_u32());
        self.version = try!(parser.read_u8());
        self.flags = try!(parser.read_flags());
        self.component_type = try!(parser.read_u32());
        self.component_subtype = try!(parser.read_u32());

        Ok(())
    }
}
