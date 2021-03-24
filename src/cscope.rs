use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::path::{Path, PathBuf};

// Resources:
// The cscope database format is internal to cscope and is not published.
// I did find an older man page published with the format data, so that is
// what we are going on in the parse below.  The comments in angle brackets are
// from the aforementioned older man page:
// https://codecat.tistory.com/entry/cscope-manpage

#[derive(Debug)]
struct Cscope {
    version: u32,
    current_dir: PathBuf,
    trailer_offset: u32,
}

#[repr(u8)]
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

    let trailer: u32;
    let offset = words.last().unwrap().trim_start_matches("0").trim_end();
    match offset.parse() {
        Ok(t) => trailer = t,
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    }

    Ok(Cscope {
        version: ver,
        current_dir: path,
        trailer_offset: trailer,
    })
}

fn parse_file_mark(fp: &mut BufReader<File>) -> Result<FileMark, Error> {
    // Read in the tab character
    let mut ch: [u8; 1] = [0];
    if let Err(e) = fp.read(&mut ch) {
        return Err(e);
    }
    if ch[0] != '\t' as u8 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Expected tab character.",
        ));
    }

    // Read the mark character.
    if let Err(e) = fp.read(&mut ch) {
        return Err(e);
    }
    Ok(ch[0].into())
}

fn parse_file_path(fp: &mut BufReader<File>) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![];
    match fp.read_until('\n' as u8, &mut buf) {
        Ok(_) => Ok(std::str::from_utf8(&buf).unwrap().to_string()),
        Err(e) => Err(e),
    }
}

fn parse_empty_line(fp: &mut BufReader<File>) -> Result<(), Error> {
    let mut ch: [u8; 1] = [0];
    if let Err(e) = fp.read(&mut ch) {
        return Err(e);
    }

    if ch[0] as char != '\n' {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Expected newline/empty_line.",
        ));
    }
    Ok(())
}

fn parse_line_number_and_blank(fp: &mut BufReader<File>) -> Result<u32, Error> {
    let mut buf: Vec<u8> = vec![];
    let line: String;
    // Read up to the blank, thus consuming the blank character (space).
    match fp.read_until(' ' as u8, &mut buf) {
        Ok(_) => line = std::str::from_utf8(&buf).unwrap().to_string(),
        Err(e) => return Err(e),
    }

    match line.parse() {
        Ok(n) => Ok(n),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "Failed to parse line number.",
        )),
    }
}

fn parse_to_end(fp: &mut BufReader<File>) -> Result<String, Error> {
    // TODO
    Ok("todo".to_string())
}

fn parse_optional_mark(fp: &mut BufReader<File>) -> Result<(), Error> {
    // TODO
    Ok(())
}

fn parse_symbol(fp: &mut BufReader<File>) -> Result<String, Error> {
    // TODO
    Ok("todo".to_string())
}

fn parse_until_empty_line(fp: &mut BufReader<File>) -> Result<String, Error> {
    // TODO
    Ok("todo".to_string())
}

// Parse the symbols for a file.
fn parse_symbol_data(fp: &mut BufReader<File>, _cscope: &mut Cscope) -> Result<(), Error> {
    // <file mark> <file path>
    let _mark: FileMark;
    match parse_file_mark(fp) {
        Ok(m) => _mark = m,
        Err(e) => return Err(e),
    }
    let _fname: String;
    match parse_file_path(fp) {
        Ok(f) => _fname = f,
        Err(e) => return Err(e),
    }

    // <empty line>
    if let Err(e) = parse_empty_line(fp) {
        return Err(e);
    }

    // For each source line. (Should have used a parser combinator for this...)
    loop {
        // <line number> <blank> <non-symbol text>
        let line: u32;
        match parse_line_number_and_blank(fp) {
            Ok(number) => line = number,
            Err(e) => return Err(e),
        }
        let non_sym_text1: String;
        match parse_to_end(fp) {
            Ok(t) => non_sym_text1 = t,
            Err(e) => return Err(e),
        }

        // <optional mark> <symbol>
        let symbol: String;
        if let Err(e) = parse_optional_mark(fp) {
            return Err(e);
        }
        match parse_symbol(fp) {
            Ok(s) => symbol = s,
            Err(e) => return Err(e),
        }

        // <non-symbol text>
        let non_sym_text2: String;
        match parse_until_empty_line(fp) {
            Ok(t) => non_sym_text2 = t,
            Err(e) => return Err(e),
        }
        break;
    }

    Ok(())
}

fn parse_body(fp: &mut BufReader<File>, cscope: &mut Cscope) -> Result<(), Error> {
    // Parse the symbol data until we reach the trailer.
    loop {
        if let Err(e) = parse_symbol_data(fp, cscope) {
            return Err(e);
        }
        break;
    }
    Ok(())
}

fn test_foo() -> Result<(), Error>
{
    Err(Error::new(ErrorKind::NotFound, "Foo"))
}

pub fn parse_database(filename: &Path) -> Result<(), Error> {
    let mut fp: BufReader<File>;
    match File::open(filename) {
        Ok(f) => fp = BufReader::new(f),
        Err(err) => return Err(err),
    }

    let mut cscope: Cscope;
    match parse_header(&mut fp) {
        Ok(cs) => cscope = cs,
        Err(err) => return Err(err),
    }

    if let Err(err) = parse_body(&mut fp, &mut cscope) {
        return Err(err);
    }

    println!("{:?}", cscope);
    Ok(())
}
