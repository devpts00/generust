use std::io::{BufWriter, Result, Write};

use rand::Rng;
use regex::Regex;
use structopt::StructOpt;
use uuid::Uuid;
use glob;

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

struct Elements {
    elements: Vec<String>
}

impl Generust for Elements {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let i = rand::thread_rng().gen_range(0, self.elements.len());
        w.write(self.elements[i].as_bytes()).map(|_| ())
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

        if text.eq("UUID") {
            Box::new(Uuid4 {})
        } else if text.eq("IPADDRESS") {
            Box::new(IpAddress {})
        } else if text.eq("TIMESTAMP") {
            Box::new(Timestamp {})
        } else if text.eq("BOOLEAN") {
            Box::new(Elements { elements: vec![String::from("true"), String::from("false")] })
        } else if text.eq("GENDER") {
            Box::new(Elements { elements: vec![String::from("Male"), String::from("Female")] })
        } else if text.eq("TIMEZONE") {
            let mut es = vec![];
            let tzs = glob::glob("/usr/share/zoneinfo/posix/**/*")
                .expect("unable to read timezones");
            for tz in tzs {
                if let Ok(path) = tz {
                    if path.is_file() {
                        if let Some(name) = path.file_name() {
                            es.push(name.to_os_string().into_string().unwrap())
                        }
                    }
                }
            }
            Box::new(Elements { elements: es })
        } else if let Some(cap) = rx_choice.captures(text) {
            let mut es = vec![];
            for e in cap.get(1).unwrap().as_str().split(",") {
                let te = e.trim();
                if !te.is_empty() {
                    es.push(String::from(te));
                }
            }
            Box::new(Elements { elements: es })
        } else if let Some(cap) = rx_integer.captures(text) {
            Box::new(Integer {
                min: cap.get(1).unwrap().as_str().parse::<i64>().unwrap(),
                max: cap.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            })
        } else {
            Box::new(Text{ text: String::from(text) })
        }
    }

    fn new(text: &str) -> Option<DG> {
        let mut gs: Vec<Box<dyn Generust>> = vec![];
        let mut start = 0;
        let rx = Regex::new(r"(\$\{([^}]+)})").unwrap();
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

        Some(Box::new(Composite { generusts: gs }))
    }
}

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use uuid::Uuid;

    use crate::{Composite, Generust, Integer, Text, Timestamp, Uuid4};

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
        let gr = Text { text: String::from(text) };
        assert!(gr.generate(& mut buf).is_ok());
        let str = str(buf);
        println!("text: {}", str);
        assert_eq!(text, str);
    }

    #[test]
    fn test_uuid() {
        let mut buf = buf(32);
        let gr = Uuid4::new("UUID").unwrap();
        assert!(gr.generate(&mut buf).is_ok());
        let str = str(buf);
        assert!(Uuid::parse_str(&str).is_ok());
        println!("uuid4: {}", str);
    }

    #[test]
    fn test_integer() {
        let mut buf = buf(16);
        let gr = Integer::new("INTEGER(0,100)").unwrap();
        assert!(gr.generate(& mut buf).is_ok());
        let str = str(buf);
        assert!(str.parse::<i64>().is_ok());
        println!("int: {}", str);

    }

    #[test]
    fn test_timestamp() {
        let mut buf = buf(32);
        let gr = Timestamp {};
        assert!(gr.generate(& mut buf).is_ok());
        let str = str(buf);
        assert!(DateTime::parse_from_rfc3339(&str).is_ok());
        println!("timestamp: {}", str);
    }

    #[test]
    fn test_composite() {
        let mut buf = buf(128);
        let gr = Composite {
            generusts: vec![
                Box::new(Integer {min: 0, max: 10}),
                Box::new(Text { text: String::from(", ")}),
                Box::new(Timestamp{}),
                Box::new(Text { text: String::from(", ")}),
                Box::new(Uuid4{}),
            ]
        };
        assert!(gr.generate(& mut buf).is_ok());
        let str = str(buf);
        println!("composite: {}", str);
    }

    #[test]
    fn test_parse() {
        let mut buf = buf(128);
        let gr = Composite::new("{ uuid: ${UUID}, timestamp: ${TIMESTAMP}, integer: ${INTEGER(0,100)} }").unwrap();
        for _ in 0..5 {
            assert!(gr.generate(& mut buf).is_ok());
        }
        println!("{}", str(buf));
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
}

fn main() {
    let opts: Options = Options::from_args();

    let output = std::fs::File::create(opts.output)
        .expect("unable to open output file");

    let mut buffer = BufWriter::new(output);

    let template = std::fs::read_to_string(opts.template)
        .expect("unable to read template file");

    let generust = Composite::new(&template)
        .expect("unable to parse template");

    for _ in 0..opts.count {
        generust.generate(&mut buffer).expect("unable to generate data");
    }

    buffer.flush().expect("unable to flush")
}
