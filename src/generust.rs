use std::io::{BufRead, BufReader, Result, Write};
use glob;
use memmap::{Mmap, MmapOptions};
use rand::Rng;
use regex::Regex;
use uuid::Uuid;

/// interface

pub trait Generust {
    fn generate(&self, w: &mut dyn Write) -> Result<()>;
}

/// implementation

struct Text {
    text: String
}

impl Generust for Text {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        w.write(self.text.as_bytes()).map(|_| ())
    }
}

struct Uuid4 {}

impl Generust for Uuid4 {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", Uuid::new_v4())
    }
}

struct Integer {
    min: i64,
    max: i64
}

impl Generust for Integer {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", rand::thread_rng().gen_range(self.min, self.max))
    }
}

struct IpAddress {}

impl Generust for IpAddress {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let b1 = rng.gen_range(1, 255);
        let b2 = rng.gen_range(1, 255);
        let b3 = rng.gen_range(1, 255);
        let b4 = rng.gen_range(1, 255);
        write!(w, "{}.{}.{}.{}", b1, b2, b3, b4)
    }
}

struct Timestamp {}

impl Generust for Timestamp {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", chrono::Utc::now().format("%+"))
    }
}

struct Choice {
    vars: Vec<String>
}

impl Generust for Choice {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let i = rand::thread_rng().gen_range(0, self.vars.len());
        w.write(self.vars[i].as_bytes()).map(|_| ())
    }
}

struct Phone {}

impl Generust for Phone {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let mut rng = rand::thread_rng();
        let x1 = rng.gen_range(1, 1000);
        let x2 = rng.gen_range(1, 1000);
        let x3 = rng.gen_range(1, 10000);
        write!(w, "8-{:03}-{:03}-{:04}", x1, x2, x3)
    }
}

struct MmapFile {
    mem: Mmap
}

impl Generust for MmapFile {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        w.write(Lines::random(&self.mem)).map(|_| ())
    }
}

struct EncodedId {
    min: usize,
    max: usize
}

impl Generust for EncodedId {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
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
        w.write(&buf).map(|_| ())
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
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        w.write(Lines::random(self.bytes)).map(|_| ())
    }
}

pub struct Composite {
    generusts: Vec<Box<dyn Generust>>
}

impl Generust for Composite {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        for g in &self.generusts {
            match g.generate(w) {
                Err(err) => return Err(err),
                _ => continue
            }
        }
        w.write(b"\n").map(|_| ())
    }
}

pub struct Parser {
    rx_template: Regex,
    rx_choice: Regex,
    rx_integer: Regex,
    rx_encoded_id: Regex,
    rx_file: Regex,
}

type ParserResult = std::result::Result<Parser, regex::Error>;

impl Parser {

    pub fn new(symbol: &str) -> ParserResult {
        let txt = &format!("({}{})", symbol, r"\{([^}]+)}");
        match Regex::new(txt) {
            Ok(rx) => Ok(
                Parser {
                    rx_template: rx,
                    rx_choice: Regex::new(r"^CHOICE\((.+)\)$").unwrap(),
                    rx_integer: Regex::new(r"^INTEGER\((-?\d+),(-?\d+)\)$").unwrap(),
                    rx_encoded_id: Regex::new(r"^ENCODEDID\((\d+),(\d+)\)$").unwrap(),
                    rx_file: Regex::new(r"^FILE\((.+)\)$").unwrap()
                }
            ),
            Err(err) => Err(err)
        }
    }

    pub fn parse(&self, template: &str) -> Result<Box<dyn Generust>> {
        self.parse_template(template)
    }

    fn parse_text(&self, text: &str) -> Box<dyn Generust> {
        Box::new(Text { text: String::from(text) })
    }

    fn parse_macro(&self, text: &str) -> Result<Box<dyn Generust>> {
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
            let tzs = glob::glob("/usr/share/zoneinfo/posix/**/*").unwrap();
            //let tzs = glob::glob("/root/**/*").unwrap();
            let mut vs = vec![];
            for tz in tzs {
                match tz {
                    Ok(path) => {
                        if path.is_file() {
                            if let Some(name) = path.file_name() {
                                vs.push(name.to_os_string().into_string().unwrap())
                            }
                        }
                    },
                    Err(err) => {
                        return Err(err.into_error());
                    }
                }
            }
            Ok(Box::new(Choice { vars: vs }))
        } else if let Some(cap) = self.rx_file.captures(text) {
            let mut vs = vec![];
            let name = cap.get(1).unwrap().as_str().trim();
            let meta = match std::fs::metadata(name) {
                Ok(m) => m,
                Err(err) => return Err(err)
            };
            let file = match std::fs::File::open(name) {
                Ok(f) => f,
                Err(err) => return Err(err)
            };
            if meta.len() < 8 * 1024 {
                for line in BufReader::new(file).lines() {
                    match line {
                        Ok(txt) => vs.push(txt),
                        Err(err) => return Err(err)
                    }
                }
                Ok(Box::new(Choice{vars: vs}))
            } else {
                let mmap = unsafe {
                    match MmapOptions::new().map(&file) {
                        Ok(m) => m,
                        Err(err) => return Err(err)
                    }
                };
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

    fn parse_template(&self, template: &str) -> Result<Box<dyn Generust>> {
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
            match self.parse_macro(inner.as_str()) {
                Ok(g) => gs.push(g),
                Err(err) => return Err(err)
            }

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
