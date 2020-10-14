use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::num::ParseIntError;
use std::option;

use chrono::{Duration, Local, NaiveDate, NaiveDateTime, ParseError};
use memmap::{Mmap, MmapOptions};
use rand::Rng;
use regex::Regex;
use std::ffi::OsString;
use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    Macro(String),
    OsString(OsString),
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
            Error::OsString(str) => Display::fmt(str.to_string_lossy().as_ref(), f),
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

impl From<OsString> for Error {
    fn from(str: OsString) -> Self {
        Error::OsString(str)
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
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()>;
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
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(self.text.as_bytes()).map(|_| ())?)
    }
}

struct RowNum {
    start: i32,
}

impl RowNum {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let start: i32 = match args.len() {
            0 => 0,
            1 => args[0].parse::<i32>()?,
            _ => return Err(Error::Macro("ROW_NUM".to_string())),
        };
        Ok(Box::new(RowNum { start }))
    }
}

impl Generust for RowNum {
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", self.start + i)?)
    }
}

struct DateRnd {
    start: i64,
    end: i64,
}

impl DateRnd {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        fn seconds(date: &str) -> Result<i64> {
            Ok(date.parse::<NaiveDate>()?.and_hms(0, 0, 0).timestamp())
        }
        let now: i64 = Local::now().timestamp();
        let (start, end) = match args.len() {
            0 => (0, now),
            2 => (seconds(args[0])?, seconds(args[1])?),
            _ => return Err(Error::Macro("DATE_RND".to_string())),
        };
        Ok(Box::new(DateRnd { start, end }))
    }
}

impl Generust for DateRnd {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let ts = rng.gen_range(self.start, self.end + 1);
        let date = NaiveDateTime::from_timestamp(ts, 0).date();
        Ok(write!(w, "{}", date)?)
    }
}

struct DateSeq {
    start: NaiveDate,
    length: i64,
}

impl DateSeq {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let (start, end) = match args.len() {
            0 => (
                NaiveDate::from_ymd(1970, 1, 1),
                Local::now().naive_local().date(),
            ),
            2 => (args[0].parse()?, args[1].parse()?),
            _ => return Err(Error::Macro("DATE_SEQ".to_string())),
        };
        let length = (end - start).num_days();
        Ok(Box::new(DateSeq { start, length }))
    }
}

impl Generust for DateSeq {
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()> {
        let date = self.start + Duration::days(i as i64 % self.length * self.length.signum());
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
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", Uuid::new_v4())?)
    }
}

struct IntSeq {
    start: i32,
    end: i32,
}

impl IntSeq {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let (start, end) = match args.len() {
            0 => (0, std::i32::MAX),
            1 => (0, args[0].parse()?),
            2 => (args[0].parse()?, args[1].parse()?),
            _ => return Err(Error::Macro("INT_SEQ".to_string())),
        };
        Ok(Box::new(IntSeq { start, end }))
    }
}

impl Generust for IntSeq {
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", self.start + i % (self.end - self.start))?)
    }
}

struct IntRnd {
    start: i32,
    end: i32,
}

impl IntRnd {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let (start, end) = match args.len() {
            0 => (0, std::i32::MAX),
            1 => (0, args[0].parse()?),
            2 => (args[0].parse()?, args[1].parse()?),
            _ => return Err(Error::Macro("INT_RND".to_string())),
        };
        Ok(Box::new(IntRnd { start, end }))
    }
}

impl Generust for IntRnd {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        Ok(write!(w, "{}", rng.gen_range(self.start, self.end))?)
    }
}

struct IpV4;

impl IpV4 {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(IpV4))
    }
}

impl Generust for IpV4 {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
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
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(write!(w, "{}", chrono::Utc::now().format("%+"))?)
    }
}

struct EnumRnd {
    vars: Vec<String>,
}

impl EnumRnd {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let vars = match args.len() {
            0 => return Err(Error::Macro("ENUM_RND".to_string())),
            _ => args.iter().map(|v| v.to_string()).collect::<Vec<String>>(),
        };
        Ok(Box::new(EnumRnd { vars }))
    }
    fn create_boolean(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(EnumRnd {
            vars: vec!["true".to_string(), "false".to_string()],
        }))
    }
    fn create_gender(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(EnumRnd {
            vars: vec!["Male".to_string(), "Female".to_string()],
        }))
    }
    fn create_time_zone(_args: &[&str]) -> Result<Box<dyn Generust>> {
        let tzs = glob::glob("/usr/share/zoneinfo/posix/**/*")?;
        let mut vs = vec![];
        for tz in tzs {
            let path = tz?;
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    vs.push(name.to_os_string().into_string()?)
                }
            }
        }
        Ok(Box::new(EnumRnd { vars: vs }))
    }
}

impl Generust for EnumRnd {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0, self.vars.len());
        Ok(w.write(self.vars[i].as_bytes()).map(|_| ())?)
    }
}

struct EnumSeq {
    vars: Vec<String>,
}

impl EnumSeq {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let vars = match args.len() {
            0 => return Err(Error::Macro("ENUM_SEQ".to_string())),
            _ => args.iter().map(|v| v.to_string()).collect::<Vec<String>>(),
        };
        Ok(Box::new(EnumSeq { vars }))
    }
}

impl Generust for EnumSeq {
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(self.vars[i as usize % self.vars.len()].as_bytes())
            .map(|_| ())?)
    }
}

struct Phone;

impl Phone {
    fn create(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(Phone))
    }
}

impl Generust for Phone {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
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

static BYTES_FIRST: &[u8] = include_bytes!("resources/first");
static BYTES_LAST: &[u8] = include_bytes!("resources/last");
static BYTES_DOMAIN: &[u8] = include_bytes!("resources/domains");
static BYTES_COUNTRY_CODES: &[u8] = include_bytes!("resources/country_codes");

struct BytesRnd<'a> {
    bytes: &'a [u8],
}

impl BytesRnd<'_> {
    fn create_first(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesRnd { bytes: BYTES_FIRST }))
    }
    fn create_last(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesRnd { bytes: BYTES_LAST }))
    }
    fn create_domain(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesRnd {
            bytes: BYTES_DOMAIN,
        }))
    }
    fn create_country_code(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesRnd {
            bytes: BYTES_COUNTRY_CODES,
        }))
    }
}

impl<'a> Generust for BytesRnd<'a> {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(random_line(self.bytes)).map(|_| ())?)
    }
}

fn next_line<'a>(data: &'a [u8], offset: &mut usize) -> &'a [u8] {
    let start = *offset;
    let mut end = *offset;
    while end < data.len() && data[end] != b'\n' {
        end += 1;
    }
    *offset = end + 1;
    if *offset >= data.len() {
        *offset = 0;
    }
    &data[start..end]
}

struct BytesSeq<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl BytesSeq<'_> {
    fn create_first(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesSeq {
            bytes: BYTES_FIRST,
            offset: 0,
        }))
    }
    fn create_last(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesSeq {
            bytes: BYTES_LAST,
            offset: 0,
        }))
    }
    fn create_domain(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesSeq {
            bytes: BYTES_DOMAIN,
            offset: 0,
        }))
    }
    fn create_country_code(_args: &[&str]) -> Result<Box<dyn Generust>> {
        Ok(Box::new(BytesSeq {
            bytes: BYTES_COUNTRY_CODES,
            offset: 0,
        }))
    }
}

impl<'a> Generust for BytesSeq<'a> {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(next_line(self.bytes, &mut self.offset))
            .map(|_| ())?)
    }
}

struct FileRnd {
    mem: Mmap,
}

impl FileRnd {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let name = match args.len() {
            1 => args[0],
            _ => return Err(Error::Macro("FILE_RND".to_string())),
        };
        let file = std::fs::File::open(name)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Box::new(FileRnd { mem: mmap }))
    }
}

impl Generust for FileRnd {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(random_line(&self.mem)).map(|_| ())?)
    }
}

struct FileSeq {
    mem: Mmap,
    offset: usize,
}

impl FileSeq {
    fn create(args: &[&str]) -> Result<Box<dyn Generust>> {
        let name = match args.len() {
            1 => args[0],
            _ => return Err(Error::Macro("FILE_SEQ".to_string())),
        };
        let file = std::fs::File::open(name)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Box::new(FileSeq {
            mem: mmap,
            offset: 0,
        }))
    }
}

impl Generust for FileSeq {
    fn generate(&mut self, _i: i32, w: &mut dyn Write) -> Result<()> {
        Ok(w.write(next_line(&self.mem, &mut self.offset))
            .map(|_| ())?)
    }
}

pub struct Composite {
    generusts: Vec<Box<dyn Generust>>,
}

impl Generust for Composite {
    fn generate(&mut self, i: i32, w: &mut dyn Write) -> Result<()> {
        for g in &mut self.generusts {
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
    separator_args: String,
}

impl Parser {
    pub fn new(macro_start: &str, separator_args: &str) -> Result<Parser> {
        let rx_template = Regex::new(&format!("({}{})", macro_start, r"\{([^}]+)}"))?;
        let rx_macro = Regex::new(r"(^.+)\((.*)\)")?;

        fn reg(fs: &mut HashMap<String, MacroFactory>, name: &str, f: MacroFactory) {
            fs.insert(name.to_string(), f);
        }

        let mut mc_factories = HashMap::new();

        reg(&mut mc_factories, "ROW_NUM", RowNum::create);
        reg(&mut mc_factories, "INT_SEQ", IntSeq::create);
        reg(&mut mc_factories, "INT_RND", IntRnd::create);
        reg(&mut mc_factories, "DATE_SEQ", DateSeq::create);
        reg(&mut mc_factories, "DATE_RND", DateRnd::create);
        reg(&mut mc_factories, "UUID4", Uuid4::create);
        reg(&mut mc_factories, "IPV4", IpV4::create);
        reg(&mut mc_factories, "TIMESTAMP", Timestamp::create);
        reg(&mut mc_factories, "ENUM_SEQ", EnumSeq::create);
        reg(&mut mc_factories, "ENUM_RND", EnumRnd::create);
        reg(&mut mc_factories, "TIME_ZONE", EnumRnd::create_time_zone);
        reg(&mut mc_factories, "BOOLEAN", EnumRnd::create_boolean);
        reg(&mut mc_factories, "GENDER", EnumRnd::create_gender);
        reg(&mut mc_factories, "PHONE", Phone::create);
        reg(&mut mc_factories, "FIRST_SEQ", BytesSeq::create_first);
        reg(&mut mc_factories, "FIRST_RND", BytesRnd::create_first);
        reg(&mut mc_factories, "LAST_SEQ", BytesSeq::create_last);
        reg(&mut mc_factories, "LAST_RND", BytesRnd::create_last);
        reg(&mut mc_factories, "DOMAIN_SEQ", BytesSeq::create_domain);
        reg(&mut mc_factories, "DOMAIN_RND", BytesRnd::create_domain);
        reg(
            &mut mc_factories,
            "COUNTRY_CODE_SEQ",
            BytesSeq::create_country_code,
        );
        reg(
            &mut mc_factories,
            "COUNTRY_CODE_RND",
            BytesRnd::create_country_code,
        );
        reg(&mut mc_factories, "FILE_RND", FileRnd::create);
        reg(&mut mc_factories, "FILE_SEQ", FileSeq::create);

        Ok(Parser {
            rx_template,
            rx_macro,
            mc_factories,
            separator_args: separator_args.to_string(),
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

    fn parse_macro(&self, text: &str) -> Result<Box<dyn Generust>> {
        let (name, args) = match self.rx_macro.captures(text) {
            Some(cap) => {
                let name = cap.get(1)?.as_str();
                let args = cap
                    .get(2)?
                    .as_str()
                    .split(&self.separator_args)
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
            gs.push(self.parse_macro(inner.as_str())?);

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

    extern crate test;

    use std::net::Ipv4Addr;

    use chrono::{DateTime, NaiveDate};
    use uuid::Uuid;

    use crate::generust::{Generust, Parser};
    use test::Bencher;

    fn parser() -> Parser {
        Parser::new("\\$", ",").unwrap()
    }

    fn parse(name: &str) -> Box<dyn Generust> {
        parser().parse_macro(name).unwrap()
    }

    fn generate(g: &mut Box<dyn Generust>, i: i32) -> String {
        let mut buf = Vec::with_capacity(512);
        assert!(g.generate(i, &mut buf).is_ok());
        String::from_utf8(buf).expect("invalid utf8")
    }

    type Probe = fn(i: i32, s: &str);

    fn roll(mut g: &mut Box<dyn Generust>, f: Probe) {
        for i in 0..123 {
            f(i, &generate(&mut g, i));
        }
    }

    #[test]
    fn test_text() {
        let s1 = "hello";
        let mut g = parser().parse_text(s1);
        let s2 = generate(&mut g, 0);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_row_num() {
        let mut g = parse("ROW_NUM");
        roll(&mut g, |i, s| {
            assert_eq!(i, s.parse().unwrap());
        });
    }

    #[test]
    fn test_int_seq() {
        let mut g = parse("INT_SEQ(3, 17)");
        roll(&mut g, |_, s| assert!(s.parse::<i32>().is_ok()));
    }

    #[test]
    fn test_int_rnd() {
        let mut g = parse("INT_RND(3, 17)");
        roll(&mut g, |_, s| assert!(s.parse::<i32>().is_ok()));
    }

    #[test]
    fn test_date_seq() {
        let mut g = parse("DATE_SEQ(2010-01-01,2020-02-02)");
        roll(&mut g, |_, s| {
            assert!(s.parse::<NaiveDate>().is_ok());
        });
    }

    #[test]
    fn test_date_rnd() {
        let mut g = parse("DATE_RND(2010-01-01,2020-02-02)");
        roll(&mut g, |_, s| {
            assert!(s.parse::<NaiveDate>().is_ok());
        });
    }

    #[test]
    fn test_uuid4() {
        let mut g = parse("UUID4");
        roll(&mut g, |_, s| {
            assert!(Uuid::parse_str(s).is_ok());
        });
    }

    #[test]
    fn test_ipv4() {
        let mut g = parse("IPV4");
        roll(&mut g, |_, s| {
            assert!(s.parse::<Ipv4Addr>().is_ok());
        });
    }

    #[test]
    fn test_timestamp() {
        let mut g = parse("TIMESTAMP");
        roll(&mut g, |_, s| {
            assert!(DateTime::parse_from_rfc3339(s).is_ok());
        });
    }

    #[test]
    fn test_enum_seq() {
        let mut g = parse("ENUM_SEQ(1,2,3)");
        roll(&mut g, |_, s| {
            assert!(s.parse::<i32>().is_ok());
        });
    }

    #[test]
    fn test_enum_rnd() {
        let mut g = parse("ENUM_RND(1,2,3)");
        roll(&mut g, |_, s| {
            assert!(s.parse::<i32>().is_ok());
        });
    }

    #[test]
    fn test_timezone() {
        let mut g = parse("TIME_ZONE");
        roll(&mut g, |_, s| {
            assert!(!s.is_empty());
        });
    }

    #[test]
    fn test_boolean() {
        let mut g = parse("BOOLEAN");
        roll(&mut g, |_, s| {
            assert!(s.eq("true") || s.eq("false"));
        });
    }

    #[test]
    fn test_gender() {
        let mut g = parse("GENDER");
        roll(&mut g, |_, s| {
            assert!(s.eq("Male") || s.eq("Female"));
        });
    }

    #[test]
    fn test_phone() {
        let mut g = parse("PHONE");
        roll(&mut g, |_, s| {
            assert!(!s.is_empty());
        });
    }

    #[test]
    fn test_bytes_seq() {
        let mut g = parse("FIRST_SEQ");
        roll(&mut g, |_, s| {
            assert!(!s.is_empty());
        });
    }

    #[test]
    fn test_bytes_rnd() {
        let mut g = parse("FIRST_RND");
        roll(&mut g, |_, s| {
            assert!(!s.is_empty());
        });
    }

    #[test]
    fn test_composite() {
        let mut g = parser()
            .parse("@{UUID4},@{ENUM_SEQ(1,2,3),@{INT_RND(1,10)}")
            .unwrap();
        let mut buf = Vec::with_capacity(128);
        assert!(g.generate(0, &mut buf).is_ok());
    }

    #[bench]
    fn bench_bytes_rnd(b: &mut Bencher) {
        b.iter(|| test_bytes_rnd())
    }
}
