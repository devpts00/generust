use std::io::{BufWriter, Result, Write, BufReader, BufRead};

use glob;
use rand::Rng;
use regex::Regex;
use structopt::StructOpt;
use uuid::Uuid;
use memmap::{MmapOptions, Mmap};

trait Generust {
    fn generate(&self, w: &mut dyn Write) -> Result<()>;
}

type DG = Box<dyn Generust>;

struct Text {
    text: String
}

impl Generust for Text {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        w.write(self.text.as_bytes()).map(|_| ())
    }
}

impl Text {
    fn new(text: &str) -> Option<DG> {
        if text.is_empty() {
            return None;
        }
        Some(Box::new(
            Text { text: String::from(text) }
        ))
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
        let offset = rand::thread_rng().gen_range(0, self.mem.len());
        let mut start = offset;
        while start > 0 && self.mem[start - 1] != b'\n' {
            start -= 1;
        }
        let mut end = offset;
        while end < self.mem.len() && self.mem[end] != b'\n' {
            end += 1;
        }
        w.write(&self.mem[start..end]).map(|_| ())
    }
}

struct Composite {
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

impl Composite {

    fn create(text: &str) -> Box<dyn Generust> {

        let rx_choice = Regex::new(r"^CHOICE\((.+)\)$").unwrap();
        let rx_integer = Regex::new(r"^INTEGER\((-?\d+),(-?\d+)\)$").unwrap();
        let rx_file = Regex::new(r"^FILE\((.+)\)$").unwrap();

        if text.eq("UUID") {
            Box::new(Uuid4 {})
        } else if text.eq("IPADDRESS") {
            Box::new(IpAddress {})
        } else if text.eq("TIMESTAMP") {
            Box::new(Timestamp {})
        } else if text.eq("PHONE") {
            Box::new(Phone {})
        } else if text.eq("BOOLEAN") {
            Box::new(Choice { vars: vec![String::from("true"), String::from("false")] })
        } else if text.eq("GENDER") {
            Box::new(Choice { vars: vec![String::from("Male"), String::from("Female")] })
        } else if text.eq("TIMEZONE") {
            let mut vs = vec![];
            let tzs = glob::glob("/usr/share/zoneinfo/posix/**/*")
                .expect("unable to read timezones");
            for tz in tzs {
                if let Ok(path) = tz {
                    if path.is_file() {
                        if let Some(name) = path.file_name() {
                            vs.push(name.to_os_string().into_string().unwrap())
                        }
                    }
                }
            }
            Box::new(Choice { vars: vs })
        } else if let Some(cap) = rx_file.captures(text) {
            let mut vs = vec![];
            let name = cap.get(1).unwrap().as_str().trim();

            let meta = std::fs::metadata(name)
                .expect("unable to get file info");
            let file = std::fs::File::open(name)
                .expect("unable to open file");
            if meta.len() < 8 * 1024 {
                for line in BufReader::new(file).lines() {
                    match line {
                        Ok(txt) => vs.push(txt),
                        Err(err) => panic!("failed to read line: {}", err)
                    }
                }
                Box::new(Choice{vars: vs})
            } else {
                println!("mmap!");
                let mmap = unsafe {
                    MmapOptions::new().map(&file).expect("unable to mmap file")
                };

                Box::new(MmapFile{ mem: mmap })
            }
        } else if let Some(cap) = rx_choice.captures(text) {
            let mut vs = vec![];
            for v in cap.get(1).unwrap().as_str().split(",") {
                let tv = v.trim();
                if !tv.is_empty() {
                    vs.push(String::from(tv));
                }
            }
            Box::new(Choice { vars: vs })
        } else if let Some(cap) = rx_integer.captures(text) {
            Box::new(Integer {
                min: cap.get(1).unwrap().as_str().parse::<i64>().unwrap(),
                max: cap.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            })
        } else {
            Box::new(Text{ text: String::from(text) })
        }
    }

    fn parse(text: &str, symbol: &str) -> Box<dyn Generust> {
        let mut gs: Vec<Box<dyn Generust>> = vec![];
        let mut start = 0;

        let rx = format!("({}{})", symbol, r"\{([^}]+)}");
        let rx = Regex::new(&rx).unwrap();
        for cap in rx.captures_iter(text) {

            let outer = cap.get(1).unwrap();
            let inner = cap.get(2).unwrap();

            // Text
            match Text::new(&text[start .. outer.start()]) {
                Some(gr) => gs.push(gr),
                None => (),
            }

            // Generust
            gs.push(Composite::create(inner.as_str()));

            start = outer.end();
        }

        // Text
        match Text::new(&text[start .. ]) {
            Some(g) => gs.push(g),
            None => (),
        }

        Box::new(Composite { generusts: gs })
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
    fn test_composite() {
        let g = Composite::parse("@{UUID},@{CHOICE(1,2,3),@{INTEGER(1,10)}", r"@");
        let mut buf = buf(128);
        assert!(g.generate(& mut buf).is_ok());
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Generust", author = "devpts00", about = "Data generator tool")]
struct Options {
    #[structopt(short, long, default_value = "template.txt", help = "Template file")]
    template: String,
    #[structopt(short, long, default_value = "output.txt", help = "Output file")]
    output: String,
    #[structopt(short, long, default_value = "100", help = "Number of records to generate")]
    count: u32,
    #[structopt(short, long, default_value = r"\$", help = "Symbol to start macros in regex")]
    symbol: String,
}

fn main() {
    let opts: Options = Options::from_args();

    let output = std::fs::File::create(opts.output)
        .expect("unable to open output file");

    let mut buffer = BufWriter::new(output);

    let template = std::fs::read_to_string(opts.template)
        .expect("unable to read template file");

    let generust = Composite::parse(&template, &opts.symbol);

    for _ in 0..opts.count {
        generust.generate(&mut buffer).expect("unable to generate data");
    }

    buffer.flush().expect("unable to flush")
}
