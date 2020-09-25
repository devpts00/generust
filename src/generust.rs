use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::num::ParseIntError;
use std::option;

use chrono::{Local, NaiveDate, NaiveDateTime, ParseError};
use memmap::{Mmap, MmapOptions};
use rand::Rng;
use regex::Regex;
use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    Macro(String),
    Io(std::io::Error),
    Regex(regex::Error),
    Glob(glob::GlobError),
    Pattern(glob::PatternError),
    None(option::NoneError),
    ParseInt(ParseIntError),
    ParseChrono(chrono::ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Macro(txt) => Display::fmt(txt, f),
            Error::Io(err) => Display::fmt(err, f),
            Error::Regex(err) => Display::fmt(err, f),
            Error::Glob(err) => Display::fmt(err, f),
            Error::Pattern(err) => Display::fmt(err, f),
            Error::None(err) => Debug::fmt(err, f),
            Error::ParseChrono(err) => Display::fmt(err, f),
            Error::ParseInt(err) => Display::fmt(err, f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Regex(err)
    }
}

impl From<glob::GlobError> for Error {
    fn from(err: glob::GlobError) -> Self {
        Error::Glob(err)
    }
}

impl From<glob::PatternError> for Error {
    fn from(err: glob::PatternError) -> Self {
        Error::Pattern(err)
    }
}

impl From<option::NoneError> for Error {
    fn from(err: option::NoneError) -> Self {
        Error::None(err)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseChrono(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseInt(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Generust {
    fn generate(&self, i: u32, w: &mut dyn Write) -> Result<()>;
}

struct Text {
    text: String,
}

impl Text {
    fn parse(text: &str) -> Result<Box<dyn Generust>> {
        Ok(Box::new(Text {
            text: String::from(text),
        }))
    }
}

impl Generust for Text {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(self.text.as_bytes()).map(|_| ())?)
    }
}

struct Index {
    start: i32,
}

impl Index {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let start: i32 = match args.len() {
            0 => 0,
            1 => args[0].parse::<i32>()?,
            _ => return Err(Error::Macro("INDEX - too many arguments".to_string())),
        };
        Ok(Box::new(Index { start }))
    }
}

impl Generust for Index {
    fn generate(&self, i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", self.start + i as i32)?)
    }
}

struct DateRnd {
    min: i64,
    max: i64,
}

impl DateRnd {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        fn seconds(date: &str) -> Result<i64> {
            Ok(date.parse::<NaiveDate>()?.and_hms(0, 0, 0).timestamp())
        }
        let now: i64 = Local::now().timestamp();
        let (min, max) = match args.len() {
            0 => (0, now),
            1 => (seconds(args[0])?, now),
            _ => (seconds(args[0])?, seconds(args[1])?),
        };
        Ok(Box::new(DateRnd { min, max }))
    }
}

impl Generust for DateRnd {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let ts = rng.gen_range(self.min, self.max + 1);
        let date = NaiveDateTime::from_timestamp(ts, 0).date();
        Ok(write!(w, "{}", date)?)
    }
}

struct Uuid4;

impl Uuid4 {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(Uuid4))
    }
}

impl Generust for Uuid4 {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", Uuid::new_v4())?)
    }
}

struct Integer {
    min: i64,
    max: i64,
}

impl Integer {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let (min, max) = match args.len() {
            0 => (std::i64::MIN, std::i64::MAX),
            1 => (0, args[0].parse::<i64>()?),
            _ => (args[0].parse::<i64>()?, args[1].parse::<i64>()?),
        };
        Ok(Box::new(Integer { min, max }))
    }
}

impl Generust for Integer {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        Ok(write!(w, "{}", rng.gen_range(self.min, self.max + 1))?)
    }
}

struct IpV4Address;

impl IpV4Address {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(IpV4Address))
    }
}

impl Generust for IpV4Address {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let b1 = rng.gen_range(1, 255);
        let b2 = rng.gen_range(1, 255);
        let b3 = rng.gen_range(1, 255);
        let b4 = rng.gen_range(1, 255);
        Ok(write!(w, "{}.{}.{}.{}", b1, b2, b3, b4)?)
    }
}

struct Timestamp;

impl Timestamp {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(Timestamp))
    }
}

impl Generust for Timestamp {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", chrono::Utc::now().format("%+"))?)
    }
}

struct Choice {
    vars: Vec<String>,
}

impl Choice {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        if args.is_empty() {
            return Err(Error::Macro("CHOICE - not enough arguments".to_string()));
        }
        let vars = args.iter().map(|v| v.to_string()).collect::<Vec<String>>();
        Ok(Box::new(Choice { vars }))
    }
}

impl Generust for Choice {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0, self.vars.len());
        Ok(w.write(self.vars[i].as_bytes()).map(|_| ())?)
    }
}

struct Phone;

impl Phone {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(Phone))
    }
}

impl Generust for Phone {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let x1 = rng.gen_range(1, 1000);
        let x2 = rng.gen_range(1, 1000);
        let x3 = rng.gen_range(1, 10000);
        Ok(write!(w, "8-{:03}-{:03}-{:04}", x1, x2, x3)?)
    }
}

fn random_line(data: &[u8]) -> &[u8] {
    let mut rng = rand::thread_rng();
    let offset = rng.gen_range(0, data.len());
    let mut start = offset;
    while start > 0 && data[start - 1] != b'\n' {
        start -= 1;
    }
    let mut end = offset;
    while end < data.len() && data[end] != b'\n' {
        end += 1;
    }
    &data[start..end]
}

static BYTES_FIRST: &[u8] = include_bytes!("../dat/first");
static BYTES_LAST: &[u8] = include_bytes!("../dat/last");
static BYTES_DOMAIN: &[u8] = include_bytes!("../dat/domains");
static BYTES_COUNTRY_CODES: &[u8] = include_bytes!("../dat/country_codes");

struct MemLines<'a> {
    bytes: &'a [u8],
}

impl MemLines<'_> {
    fn create_first(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(MemLines { bytes: BYTES_FIRST }))
    }
    fn create_last(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(MemLines { bytes: BYTES_LAST }))
    }
    fn create_domain(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(MemLines {
            bytes: BYTES_DOMAIN,
        }))
    }
    fn create_country_codes(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(MemLines {
            bytes: BYTES_COUNTRY_CODES,
        }))
    }
}

impl<'a> Generust for MemLines<'a> {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(random_line(self.bytes)).map(|_| ())?)
    }
}

struct MmapFile {
    mem: Mmap,
}

impl MmapFile {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        if args.len() != 1 {
            return Err(Error::Macro("FILE - expects one argument".to_string()));
        }
        let name = args[0];
        let file = std::fs::File::open(name)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Box::new(MmapFile { mem: mmap }))
    }
}

impl Generust for MmapFile {
    fn generate(&self, _i: u32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(random_line(&self.mem)).map(|_| ())?)
    }
}

pub struct Composite {
    generusts: Vec<Box<dyn Generust>>,
}

impl Generust for Composite {
    fn generate(&self, i: u32, w: &mut dyn Write) -> Result<()> {
        for g in &self.generusts {
            g.generate(i, w)?;
        }
        Ok(w.write(b"\n").map(|_| ())?)
    }
}

type MacroFactory = fn(&[&str]) -> Result<Box<dyn Generust>>;

pub struct Parser {
    rx_template: Regex,
    rx_macro: Regex,
    mc_factories: HashMap<String, MacroFactory>,
}

impl Parser {
    pub fn new(symbol: &str) -> Result<Parser> {
        let rx_template = Regex::new(&format!("({}{})", symbol, r"\{([^}]+)}"))?;
        let rx_macro = Regex::new(r"(^.+)\((.*)\)")?;
        let mut mc_factories = HashMap::new();
        mc_factories.insert("INDEX".to_string(), Index::create as MacroFactory);
        mc_factories.insert("DATE".to_string(), DateRnd::create as MacroFactory);
        mc_factories.insert("UUID4".to_string(), Uuid4::create as MacroFactory);
        mc_factories.insert("INTEGER".to_string(), Integer::create as MacroFactory);
        mc_factories.insert(
            "IPV4_ADDRESS".to_string(),
            IpV4Address::create as MacroFactory,
        );
        mc_factories.insert("TIMESTAMP".to_string(), Timestamp::create as MacroFactory);
        mc_factories.insert("CHOICE".to_string(), Choice::create as MacroFactory);
        mc_factories.insert("PHONE".to_string(), Phone::create as MacroFactory);
        mc_factories.insert("FILE".to_string(), MmapFile::create as MacroFactory);
        mc_factories.insert("FIRST".to_string(), MemLines::create_first as MacroFactory);
        mc_factories.insert("LAST".to_string(), MemLines::create_last as MacroFactory);
        mc_factories.insert(
            "DOMAIN".to_string(),
            MemLines::create_domain as MacroFactory,
        );
        mc_factories.insert(
            "COUNTRY_CODE".to_string(),
            MemLines::create_country_codes as MacroFactory,
        );

        Ok(Parser {
            rx_template,
            rx_macro,
            mc_factories,
        })
    }

    pub fn parse(&self, template: &str) -> Result<Box<dyn Generust>> {
        self.parse_template(template)
    }

    fn parse_text(&self, text: &str) -> Box<dyn Generust> {
        Box::new(Text {
            text: String::from(text),
        })
    }

    /*
    fn parse_macro(&self, text: &str) -> Result<Box<dyn Generust>> {
        if text.eq("INDEX") {
            Ok(Box::new(Index {}))
        } else if text.eq("UUID") {
            Ok(Box::new(Uuid4 {}))
        } else if text.eq("IPV4_ADDRESS") {
            Ok(Box::new(IpV4Address {}))
        } else if text.eq("TIMESTAMP") {
            Ok(Box::new(Timestamp {}))
        } else if text.eq("PHONE") {
            Ok(Box::new(Phone {}))
        } else if text.eq("BOOLEAN") {
            Ok(Box::new(Choice {
                vars: vec![String::from("true"), String::from("false")],
            }))
        } else if text.eq("GENDER") {
            Ok(Box::new(Choice {
                vars: vec![String::from("Male"), String::from("Female")],
            }))
        } else if text.eq("FIRST") {
            Ok(Box::new(MemLines {
                bytes: self.by_first,
            }))
        } else if text.eq("LAST") {
            Ok(Box::new(MemLines {
                bytes: self.by_last,
            }))
        } else if text.eq("DOMAIN") {
            Ok(Box::new(MemLines {
                bytes: self.by_domains,
            }))
        } else if text.eq("COUNTRY_CODE") {
            Ok(Box::new(MemLines {
                bytes: self.by_country_codes,
            }))
        } else if text.eq("TIMEZONE") {
            let tzs = glob::glob("/usr/share/zoneinfo/posix/**/*")?;
            //let tzs = glob::glob("/root/**/*").unwrap();
            let mut vs = vec![];
            for tz in tzs {
                let path = tz?;
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        vs.push(name.to_os_string().into_string().unwrap())
                    }
                }
            }
            Ok(Box::new(Choice { vars: vs }))
        } else if let Some(cap) = self.rx_file.captures(text) {
            let mut vs = vec![];

            let name = cap.get(1)?.as_str().trim();
            let meta = std::fs::metadata(name)?;
            let file = std::fs::File::open(name)?;
            if meta.len() < 8 * 1024 {
                for line in BufReader::new(file).lines() {
                    vs.push(line?);
                }
                Ok(Box::new(Choice { vars: vs }))
            } else {
                let mmap = unsafe { MmapOptions::new().map(&file)? };
                Ok(Box::new(MmapFile { mem: mmap }))
            }
        } else if let Some(cap) = self.rx_choice.captures(text) {
            let mut vs = vec![];
            for v in cap.get(1).unwrap().as_str().split(',') {
                let tv = v.trim();
                if !tv.is_empty() {
                    vs.push(String::from(tv));
                }
            }
            Ok(Box::new(Choice { vars: vs }))
        } else if let Some(cap) = self.rx_integer.captures(text) {
            let min = cap.get(1).unwrap().as_str().parse::<i64>()?;
            let max = cap.get(2).unwrap().as_str().parse::<i64>()?;
            Ok(Box::new(Integer { min, max }))
        } else if let Some(cap) = self.rx_date_range.captures(text) {
            let min = cap.get(1).unwrap().as_str().parse::<NaiveDate>()?;
            let max = cap.get(2).unwrap().as_str().parse::<NaiveDate>()?;
            Ok(Box::new(DateRnd {
                min: min.and_hms(0, 0, 0).timestamp(),
                max: max.and_hms(0, 0, 0).timestamp(),
            }))
        } else {
            Ok(self.parse_text(text))
        }
    }
     */

    fn parse_macro2(&self, text: &str) -> Result<Box<dyn Generust>> {
        let (name, args) = match self.rx_macro.captures(text) {
            Some(cap) => {
                let name = cap.get(1)?.as_str();
                let args = cap
                    .get(2)?
                    .as_str()
                    .split(',')
                    .map(|a| a.trim())
                    .collect::<Vec<&str>>();
                (name, args)
            }
            None => (text, vec![]),
        };
        match self.mc_factories.get(name) {
            Some(factory) => factory(&args),
            None => Text::parse(text),
        }
    }

    fn parse_template(&self, template: &str) -> Result<Box<dyn Generust>> {
        let mut gs: Vec<Box<dyn Generust>> = vec![];
        let mut start = 0;
        for cap in self.rx_template.captures_iter(template) {
            let outer = cap.get(1).unwrap();
            let inner = cap.get(2).unwrap();

            // Text
            if outer.start() > start {
                gs.push(self.parse_text(&template[start..outer.start()]))
            }

            // Generust
            gs.push(self.parse_macro2(inner.as_str())?);

            start = outer.end();
        }

        // Text
        if template.len() > start {
            gs.push(self.parse_text(&template[start..]))
        }

        Ok(Box::new(Composite { generusts: gs }))
    }
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;

    use chrono::DateTime;
    use uuid::Uuid;

    use crate::generust::Parser;

    fn buf(size: usize) -> Vec<u8> {
        Vec::with_capacity(size)
    }

    fn str(vec: Vec<u8>) -> String {
        String::from_utf8(vec).expect("invalid utf8")
    }

    fn parser() -> Parser {
        Parser::new("\\$").unwrap()
    }

    #[test]
    fn test_text() {
        let text = "hello";
        let mut buf = buf(10);
        let g = parser().parse_text(text);
        assert!(g.generate(0, &mut buf).is_ok());
        let str = str(buf);
        assert_eq!(text, str);
    }

    fn test(name: &str) -> String {
        let mut buf = buf(128);
        let g = parser().parse_macro(name).ok().unwrap();
        assert!(g.generate(0, &mut buf).is_ok());
        str(buf)
    }

    #[test]
    fn test_index() {
        let str = test("INDEX");
        assert!(str.parse::<u32>().is_ok())
    }

    #[test]
    fn test_uuid() {
        let str = test("UUID");
        assert!(Uuid::parse_str(&str).is_ok());
    }

    #[test]
    fn test_integer() {
        let str = test("INTEGER(0,100)");
        assert!(str.parse::<i64>().is_ok());
    }

    #[test]
    fn test_timestamp() {
        let str = test("TIMESTAMP");
        assert!(DateTime::parse_from_rfc3339(&str).is_ok());
    }

    #[test]
    fn test_ipaddress() {
        let str = test("IPADDRESS");
        assert!(str.parse::<IpAddr>().is_ok());
    }

    #[test]
    fn test_phone() {
        test("PHONE");
    }

    #[test]
    fn test_choice() {
        let str = test("CHOICE(1,2,3)");
        assert!(str.eq("1") || str.eq("2") || str.eq("3"));
    }

    #[test]
    fn test_timezone() {
        test("TIMEZONE");
    }

    #[test]
    fn test_boolean() {
        let str = test("BOOLEAN");
        assert!(str.eq("true") || str.eq("false"));
    }

    #[test]
    fn test_gender() {
        let str = test("GENDER");
        assert!(str.eq("Male") || str.eq("Female"));
    }

    #[test]
    fn test_bytes() {
        let str = test("LAST");
        assert!(!str.is_empty());
    }

    #[test]
    fn test_composite() {
        let g = parser()
            .parse("@{UUID},@{CHOICE(1,2,3),@{INTEGER(1,10)}")
            .unwrap();
        let mut buf = buf(128);
        assert!(g.generate(0, &mut buf).is_ok());
    }
}
