use std::io::{BufRead, BufReader, Write};
use glob;
use memmap::{Mmap, MmapOptions};
use rand::Rng;
use regex::Regex;
use uuid::Uuid;
use std::fmt::{Display, Formatter, Result};

pub enum GrError {
    Io(std::io::Error),
    Regex(regex::Error),
    Glob(glob::GlobError),
    Pattern(glob::PatternError)
}

impl Display for GrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            GrError::Io(err) => err.fmt(f),
            GrError::Regex(err) => err.fmt(f),
            GrError::Glob(err) => err.fmt(f),
            GrError::Pattern(err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for GrError {
    fn from(err: std::io::Error) -> Self {
        GrError::Io(err)
    }
}

impl From<regex::Error> for GrError {
    fn from(err: regex::Error) -> Self {
        GrError::Regex(err)
    }
}

impl From<glob::GlobError> for GrError {
    fn from(err: glob::GlobError) -> Self {
        GrError::Glob(err)
    }
}

impl From<glob::PatternError> for GrError {
    fn from(err: glob::PatternError) -> Self {
        GrError::Pattern(err)
    }
}

pub type GrResult<T> = std::result::Result<T, GrError>;

pub trait Generust {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()>;
}

struct Text {
    text: String
}

impl Generust for Text {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(w.write(self.text.as_bytes()).map(|_| ())?)
    }
}

struct Uuid4;

impl Generust for Uuid4 {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(write!(w, "{}", Uuid::new_v4())?)
    }
}

struct Integer {
    min: i64,
    max: i64
}

impl Generust for Integer {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(write!(w, "{}", rand::thread_rng().gen_range(self.min, self.max))?)
    }
}

struct IpAddress;

impl Generust for IpAddress {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        let mut rng = rand::thread_rng();
        let b1 = rng.gen_range(1, 255);
        let b2 = rng.gen_range(1, 255);
        let b3 = rng.gen_range(1, 255);
        let b4 = rng.gen_range(1, 255);
        Ok(write!(w, "{}.{}.{}.{}", b1, b2, b3, b4)?)
    }
}

struct Timestamp;

impl Generust for Timestamp {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(write!(w, "{}", chrono::Utc::now().format("%+"))?)
    }
}

struct Choice {
    vars: Vec<String>
}

impl Generust for Choice {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        let i = rand::thread_rng().gen_range(0, self.vars.len());
        Ok(w.write(self.vars[i].as_bytes()).map(|_| ())?)
    }
}

struct Phone;

impl Generust for Phone {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        let mut rng = rand::thread_rng();
        let x1 = rng.gen_range(1, 1000);
        let x2 = rng.gen_range(1, 1000);
        let x3 = rng.gen_range(1, 10000);
        Ok(write!(w, "8-{:03}-{:03}-{:04}", x1, x2, x3)?)
    }
}

struct MmapFile {
    mem: Mmap
}

impl Generust for MmapFile {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(w.write(Lines::random(&self.mem)).map(|_| ())?)
    }
}

struct EncodedId {
    min: usize,
    max: usize
}

impl Generust for EncodedId {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        let id = rand::thread_rng().gen_range(self.min, self.max);
        let obf = 166258;
        let rep = b"23456789BCDFGHJKLMNPQRSTVWXYZ";
        let mut id = id ^ obf;
        let mut buf = Vec::with_capacity(6);
        while id != 0 {
            buf.push(rep[id % rep.len()]);
            id = id / rep.len();
        }
        while buf.len() < 6 {
            buf.push(rep[0]);
        }
        buf.reverse();
        Ok(w.write(&buf).map(|_| ())?)
    }
}

struct Lines {
    bytes: &'static [u8]
}

impl Lines {
    fn random(data: &[u8]) -> &[u8] {
        let offset = rand::thread_rng().gen_range(0, data.len());
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
}

impl Generust for Lines {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        Ok(w.write(Lines::random(self.bytes)).map(|_| ())?)
    }
}

pub struct Composite {
    generusts: Vec<Box<dyn Generust>>
}

impl Generust for Composite {
    fn generate(&self, w: &mut dyn Write) -> GrResult<()> {
        for g in &self.generusts {
            g.generate(w)?;
        }
        Ok(w.write(b"\n").map(|_| ())?)
    }
}

pub struct Parser {
    rx_template: Regex,
    rx_choice: Regex,
    rx_integer: Regex,
    rx_encoded_id: Regex,
    rx_file: Regex,
}

impl Parser {

    pub fn new(symbol: &str) -> GrResult<Parser> {
        let txt = &format!("({}{})", symbol, r"\{([^}]+)}");
        let rx = Regex::new(txt)?;
        Ok(Parser {
            rx_template: rx,
            rx_choice: Regex::new(r"^CHOICE\((.+)\)$").unwrap(),
            rx_integer: Regex::new(r"^INTEGER\((-?\d+),(-?\d+)\)$").unwrap(),
            rx_encoded_id: Regex::new(r"^ENCODEDID\((\d+),(\d+)\)$").unwrap(),
            rx_file: Regex::new(r"^FILE\((.+)\)$").unwrap()
        })
    }

    pub fn parse(&self, template: &str) -> GrResult<Box<dyn Generust>> {
        self.parse_template(template)
    }

    fn parse_text(&self, text: &str) -> Box<dyn Generust> {
        Box::new(Text { text: String::from(text) })
    }

    fn parse_macro(&self, text: &str) -> GrResult<Box<dyn Generust>> {
        if text.eq("UUID") {
            Ok(Box::new(Uuid4 {}))
        } else if text.eq("IPADDRESS") {
            Ok(Box::new(IpAddress {}))
        } else if text.eq("TIMESTAMP") {
            Ok(Box::new(Timestamp {}))
        } else if text.eq("PHONE") {
            Ok(Box::new(Phone {}))
        } else if text.eq("BOOLEAN") {
            Ok(Box::new(Choice { vars: vec![String::from("true"), String::from("false")] }))
        } else if text.eq("GENDER") {
            Ok(Box::new(Choice { vars: vec![String::from("Male"), String::from("Female")] }))
        } else if text.eq("FIRST") {
            Ok(Box::new(Lines { bytes: include_bytes!("../dat/first.csv") }))
        } else if text.eq("LAST") {
            Ok(Box::new(Lines { bytes: include_bytes!("../dat/last.csv") }))
        } else if text.eq("DOMAIN") {
            Ok(Box::new(Lines { bytes: include_bytes!("../dat/domain.csv") }))
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
            let name = cap.get(1).unwrap().as_str().trim();
            let meta = std::fs::metadata(name)?;
            let file = std::fs::File::open(name)?;
            if meta.len() < 8 * 1024 {
                for line in BufReader::new(file).lines() {
                    vs.push(line?);
                }
                Ok(Box::new(Choice{vars: vs}))
            } else {
                let mmap = unsafe { MmapOptions::new().map(&file)? };
                Ok(Box::new(MmapFile{ mem: mmap }))
            }
        } else if let Some(cap) = self.rx_choice.captures(text) {
            let mut vs = vec![];
            for v in cap.get(1).unwrap().as_str().split(",") {
                let tv = v.trim();
                if !tv.is_empty() {
                    vs.push(String::from(tv));
                }
            }
            Ok(Box::new(Choice { vars: vs }))
        } else if let Some(cap) = self.rx_integer.captures(text) {
            Ok(Box::new(Integer {
                min: cap.get(1).unwrap().as_str().parse::<i64>().unwrap(),
                max: cap.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            }))
        } else if let Some(cap) = self.rx_encoded_id.captures(text) {
            Ok(Box::new(EncodedId {
                min: cap.get(1).unwrap().as_str().parse::<usize>().unwrap(),
                max: cap.get(2).unwrap().as_str().parse::<usize>().unwrap(),
            }))
        } else {
            Ok(self.parse_text(text))
        }
    }

    fn parse_template(&self, template: &str) -> GrResult<Box<dyn Generust>> {
        let mut gs: Vec<Box<dyn Generust>> = vec![];
        let mut start = 0;
        for cap in self.rx_template.captures_iter(template) {

            let outer = cap.get(1).unwrap();
            let inner = cap.get(2).unwrap();

            // Text
            if outer.start() > start {
                gs.push(self.parse_text(&template[start .. outer.start()]))
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
    use std::net::IpAddr;

    use chrono::DateTime;
    use uuid::Uuid;

    use crate::Composite;

    fn buf(size: usize) -> Vec<u8> {
        Vec::with_capacity(size)
    }

    fn str(vec: Vec<u8>) -> String {
        String::from_utf8(vec).expect("invalid utf8")
    }

    #[test]
    fn test_text() {
        let text = "hello";
        let mut buf = buf(10);
        let g = Composite::create(text);
        assert!(g.generate(& mut buf).is_ok());
        let str = str(buf);
        assert_eq!(text, str);
    }

    fn test(name: &str) -> String {
        let mut buf = buf(128);
        let g = Composite::create(name);
        assert!(g.generate(&mut buf).is_ok());
        str(buf)
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
        let g = Composite::parse("@{UUID},@{CHOICE(1,2,3),@{INTEGER(1,10)}", r"@");
        let mut buf = buf(128);
        assert!(g.generate(& mut buf).is_ok());
    }
}
