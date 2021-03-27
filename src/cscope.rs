use std::cmp::PartialEq;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

// Resources:
// The cscope database format is internal to cscope and is not published.
// I did find an older man page published with the format data, so that is
// what we are going on in the parse below.  The comments in angle brackets are
// from the aforementioned older man page:
// https://codecat.tistory.com/entry/cscope-manpage

#[derive(Debug)]
struct Symbol {
    mark: FileMark,
    filename: String,
    line_number: u64,
    name: String,
    non_sym_text1: String,
    non_sym_text2: String,
}

#[derive(Debug)]
struct Cscope {
    version: u32,
    current_dir: PathBuf,
    trailer_offset: u64,
    header_raw: String,
    symbols: Vec<Symbol>,
}

impl Cscope {
    pub fn is_compressed(&self) -> bool {
        match self.header_raw.split(" ").into_iter().find(|c| *c == "-c") {
            Some(_) => true,
            None => false,
        }
    }
}

impl std::fmt::Display for Cscope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut fname: &str = "";
        let max_len: usize = self
            .symbols
            .iter()
            .map(|s| {
                if s.mark == FileMark::FunctionDefinition {
                    s.name.len()
                } else {
                    0 as usize
                }
            })
            .max()
            .unwrap();
        for sym in self.symbols.iter() {
            if sym.filename != fname && sym.mark == FileMark::FunctionDefinition {
                fname = &sym.filename;
                write!(f, "•{}:\n╰─╮\n", fname)?;
            }
            if sym.mark == FileMark::FunctionDefinition {
                let sig = format!("{} {}", sym.non_sym_text1, sym.non_sym_text2);
                write!(
                    f,
                    "  ├ {name:<len$} {: <16}), line:{}\n",
                    sig,
                    sym.line_number,
                    name = sym.name,
                    len = max_len
                )?;
            }
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum FileMark {
    File = '@' as u8,
    FunctionDefinition = '$' as u8,
    FunctionCall = '`' as u8,
    FunctionEnd = '}' as u8,
    Define = '#' as u8,
    DefineEnd = ')' as u8,
    IncludeDirective = '~' as u8,
    DirectAssingmentIncDec = '=' as u8,
    EnumStructUnionDefinitonEnd = ';' as u8,
    ClassDefinition = 'c' as u8,
    EnumDefinition = 'e' as u8,
    OtherGlobalDefinition = 'g' as u8,
    FunctionBlockLocalDefinition = 'l' as u8,
    EnumStructUnionMemberGlobalDefinition = 'm' as u8,
    FunctionParameterDefinition = 'p' as u8,
    StructDefinition = 's' as u8,
    TypedefDefinition = 't' as u8,
    UnionDefinition = 'u' as u8,
    WTF = 0,
}

impl From<u8> for FileMark {
    fn from(val: u8) -> FileMark {
        match val as char {
            '@' => FileMark::File,
            '$' => FileMark::FunctionDefinition,
            '`' => FileMark::FunctionCall,
            '}' => FileMark::FunctionEnd,
            '#' => FileMark::Define,
            ')' => FileMark::DefineEnd,
            '~' => FileMark::IncludeDirective,
            '=' => FileMark::DirectAssingmentIncDec,
            ';' => FileMark::EnumStructUnionDefinitonEnd,
            'c' => FileMark::ClassDefinition,
            'e' => FileMark::EnumDefinition,
            'g' => FileMark::OtherGlobalDefinition,
            'l' => FileMark::FunctionBlockLocalDefinition,
            'm' => FileMark::EnumStructUnionMemberGlobalDefinition,
            'p' => FileMark::FunctionParameterDefinition,
            's' => FileMark::StructDefinition,
            't' => FileMark::TypedefDefinition,
            'u' => FileMark::UnionDefinition,
            _ => FileMark::WTF,
        }
    }
}

fn parse_header(fp: &mut BufReader<File>) -> Result<Cscope, Error> {
    let header: String;
    let mut buf: Vec<u8> = vec![];
    match fp.read_until('\n' as u8, &mut buf) {
        Ok(_) => match std::str::from_utf8(&buf) {
            Ok(s) => header = s.to_string(),
            Err(_) => return Err(Error::new(ErrorKind::NotFound, "Invalid line data.")),
        },
        Err(err) => return Err(err),
    }

    let words: Vec<&str> = header.split(' ').collect();
    if words.len() < 4 || words[0] != "cscope" {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid header."));
    }

    let ver: u32;
    match words[1].parse() {
        Ok(v) => ver = v,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Failed to parse version.",
            ))
        }
    }

    let path: PathBuf = PathBuf::from(words[2]);
    let trailer: u64;
    let offset = words.last().unwrap().trim_start_matches("0").trim_end();
    match offset.parse() {
        Ok(t) => trailer = t,
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    }

    Ok(Cscope {
        version: ver,
        current_dir: path,
        trailer_offset: trailer,
        header_raw: header,
        symbols: vec![],
    })
}

// This consumes 2 characters: <tab><mark>
fn parse_file_mark(fp: &mut BufReader<File>) -> Result<FileMark, Error> {
    // Read in the tab character
    let mut ch: [u8; 1] = [0];
    fp.read(&mut ch)?;
    if ch[0] != '\t' as u8 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Expected tab character.",
        ));
    }

    // Read the mark character.
    fp.read(&mut ch)?;
    Ok(ch[0].into())
}

fn parse_file_path(fp: &mut BufReader<File>) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![];
    fp.read_until('\n' as u8, &mut buf)?;
    Ok(std::str::from_utf8(&buf).unwrap().trim().to_string())
}

fn parse_empty_line(fp: &mut BufReader<File>) -> Result<(), Error> {
    let mut ch: [u8; 1] = [0];
    fp.read(&mut ch)?;
    if ch[0] as char != '\n' {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Expected newline/empty_line.",
        ));
    }
    Ok(())
}

fn parse_line_number_and_blank(fp: &mut BufReader<File>) -> Result<u64, Error> {
    // Read up to the blank, thus consuming the blank character (space).
    let mut buf: Vec<u8> = vec![];
    fp.read_until(' ' as u8, &mut buf)?;
    let line = std::str::from_utf8(&buf).unwrap().to_string();

    match line.trim().parse() {
        Ok(n) => Ok(n),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "Failed to parse line number.",
        )),
    }
}

fn parse_to_end(fp: &mut BufReader<File>) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![];
    fp.read_until('\n' as u8, &mut buf)?;
    Ok(from_utf8(&buf))
}

fn peek(fp: &mut BufReader<File>) -> u8 {
    let mut ch: [u8; 1] = [0];
    let res = fp.read(&mut ch);
    if let Err(_) = fp.seek(SeekFrom::Current(-1)) {
        return 0;
    }

    match res {
        Ok(_) => ch[0],
        Err(_) => 0,
    }
}

fn parse_optional_mark(fp: &mut BufReader<File>) -> Result<Option<FileMark>, Error> {
    if peek(fp) == '\t' as u8 {
        match parse_file_mark(fp) {
            Ok(m) => return Ok(Some(m)),
            Err(e) => return Err(e),
        }
    }
    Ok(None)
}

fn from_utf8(buf: &Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(s) => s.trim().to_string(),
        Err(_) => "<invalid utf8>".to_string(),
    }
}

fn parse_until_empty_line(fp: &mut BufReader<File>) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![];
    loop {
        let num_read: usize;
        match fp.read_until('\n' as u8, &mut buf) {
            Ok(n) => num_read = n,
            Err(e) => return Err(e),
        }
        if num_read == 1 {
            return Ok(from_utf8(&buf));
        }
    }
}

fn parse_until_next_source_line(fp: &mut BufReader<File>) -> Result<Vec<String>, Error> {
    let mut lines: Vec<String> = vec![];
    while let Ok(line) = parse_until_empty_line(fp) {
        lines.push(line);
        let ch = peek(fp) as char;
        if ch.is_digit(10) {
            return Ok(lines);
        } else if at_filemark(fp) {
            return Ok(lines);
        }
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        "Failed to locate empty line.",
    ))
}

fn at_filemark(fp: &mut BufReader<File>) -> bool {
    let found: bool;
    let idx = fp.seek(SeekFrom::Current(0)).unwrap_or(0);
    match parse_optional_mark(fp) {
        Ok(opt) => match opt {
            Some(m) => found = m == FileMark::File,
            None => found = false,
        },
        Err(_) => found = false,
    }
    match fp.seek(SeekFrom::Start(idx)) {
        Ok(_) => found,
        Err(_) => false,
    }
}

// Parse the symbols for a file.
fn parse_symbol_data(fp: &mut BufReader<File>, cscope: &mut Cscope) -> Result<(), Error> {
    // <file mark> <file path>
    let mut mark = parse_file_mark(fp)?;
    if mark != FileMark::File {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Failed to find file marker.",
        ));
    }
    let fname = parse_file_path(fp)?;

    // <empty line>
    parse_empty_line(fp)?;

    // For each source line. (Should have used a parser combinator for this...)
    while fp.seek(SeekFrom::Current(0))? < cscope.trailer_offset {
        if at_filemark(fp) {
            break;
        }
        // <line number> <blank> <non-symbol text>
        let line_number = parse_line_number_and_blank(fp)?;
        let mut non_sym_text1 = parse_to_end(fp)?;
        non_sym_text1.retain(|c| c != '\n');

        // <optional mark> <symbol>
        match parse_optional_mark(fp)? {
            Some(m) => mark = m,
            None => mark = FileMark::WTF,
        }
        let symbol = parse_to_end(fp)?.trim().to_string();

        // <non-symbol text>
        let source = parse_until_next_source_line(fp)?;
        let mut non_sym_text2: String = if source.is_empty() {
            " ".to_string()
        } else {
            source[0].clone()
        };
        non_sym_text2.retain(|c| c != '\n');

        let sym = Symbol {
            mark: mark,
            filename: fname.clone(),
            line_number: line_number,
            name: symbol,
            non_sym_text1: non_sym_text1,
            non_sym_text2: non_sym_text2,
        };
        cscope.symbols.push(sym);

        // Stop if we reach a file marker prefix (tab character).
        // This normally is a line number but will be a tab
        // when we reach the trailer start.
        if peek(fp) == '\t' as u8 {
            break;
        }
    }
    Ok(())
}

fn parse_body(fp: &mut BufReader<File>, cscope: &mut Cscope) -> Result<(), Error> {
    // Parse the symbol data until we reach the trailer.
    while fp.seek(SeekFrom::Current(0))? < cscope.trailer_offset {
        parse_symbol_data(fp, cscope)?;
        // Stop if we are at newline before the trailer marker (just before the trailer).
        if fp.seek(SeekFrom::Current(0))? + 3 == cscope.trailer_offset {
            break;
        }
    }
    Ok(())
}

pub fn parse_database(filename: &Path) -> Result<(), Error> {
    let mut fp = BufReader::new(File::open(filename)?);
    let mut cscope = parse_header(&mut fp)?;
    if !cscope.is_compressed() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "The cscope database must not be compressed.  See the '-c' option in the cscope manpage.",
        ));
    }
    parse_body(&mut fp, &mut cscope)?;
    println!("{}", cscope);
    Ok(())
}
