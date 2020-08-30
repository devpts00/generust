use std::io::{BufWriter, Result, Write};

use rand::Rng;
use regex::Regex;
use structopt::StructOpt;
use uuid::Uuid;

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

impl Uuid4 {
    fn new(text: &str) -> Option<DG> {
        if !text.eq("UUID") {
            return None;
        }
        Some(Box::new(Uuid4{}))
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

impl Integer {
    fn new(text: &str) -> Option<DG> {
        let rx = Regex::new(r"^INTEGER\((-?\d+),(-?\d+)\)$").unwrap();
        rx.captures(text).map(|cap| {
            Box::new(Integer {
                min: cap.get(1).unwrap().as_str().parse::<i64>().unwrap(),
                max: cap.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            }) as Box<dyn Generust>
        })
    }
}

struct Timestamp {}

impl Generust for Timestamp {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", chrono::Utc::now().format("%+"))
    }
}

impl Timestamp {
    fn new(text: &str) -> Option<DG> {
        if !text.eq("TIMESTAMP") {
            return None;
        }
        Some(Box::new(Timestamp{}))
    }
}

struct Boolean {}

impl Generust for Boolean {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", rand::thread_rng().gen_bool(0.5))
    }
}

impl Boolean {
    fn new(text: &str) -> Option<DG> {
        if !text.eq("BOOLEAN") {
            return None;
        }
        Some(Box::new(Boolean{}))
    }
}

struct Gender {}

impl Generust for Gender {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let gender = match rand::thread_rng().gen_bool(0.5) {
            true => "Male",
            false => "Female",
        };
        w.write(gender.as_bytes()).map(|_| ())
    }
}

impl Gender {
    fn new(text: &str) -> Option<DG> {
        if !text.eq("GENDER") {
            return None;
        }
        Some(Box::new(Gender{}))
    }
}

struct Choice {
    elements: Vec<String>
}

impl Generust for Choice {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        let i = rand::thread_rng().gen_range(0, self.elements.len());
        w.write(self.elements[i].as_bytes()).map(|_| ())
    }
}

impl Choice {
    fn new(text: &str) -> Option<DG> {
        let rx = Regex::new(r"^CHOICE\((.+)\)$").unwrap();
        rx.captures(text).map(|cap| {
            let mut choice = Box::new(Choice { elements: vec![] });
            for el in cap.get(1).unwrap().as_str().split(",") {
                let tel = el.trim();
                if !tel.is_empty() {
                    choice.elements.push(String::from(el.trim()));
                }
            }
            choice as Box<dyn Generust>
        })
    }
}

struct Composite {
    generators: Vec<Box<dyn Generust>>
}

impl Generust for Composite {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        for gr in &self.generators {
            match gr.generate(w) {
                Err(err) => return Err(err),
                _ => continue
            }
        }
        w.write(b"\n").map(|_| ())
    }
}

impl Composite {

    fn find(fs: &[fn(&str) -> Option<DG>], text: &str) -> Option<DG> {
        match fs.iter().map(|f| f(text)).find(|x| x.is_some()) {
            Some(res) => res,
            None => None
        }
    }

    fn new(text: &str) -> Option<DG> {
        let mut grs: Vec<Box<dyn Generust>> = vec![];
        let mut start = 0;
        let rx = Regex::new(r"(\$\{([^}]+)})").unwrap();
        let factory = [
            Uuid4::new,
            Timestamp::new,
            Integer::new,
            Boolean::new,
            Gender::new,
            Choice::new
        ];
        for cap in rx.captures_iter(text) {

            let outer = cap.get(1).unwrap();
            let inner = cap.get(2).unwrap();

            // Text
            match Text::new(&text[start .. outer.start()]) {
                Some(gr) => grs.push(gr),
                None => (),
            }

            // Generust
            match Composite::find(&factory, inner.as_str()) {
                Some(gr) => grs.push(gr),
                None => return None,
            }

            start = outer.end();
        }

        // Text
        match Text::new(&text[start .. ]) {
            Some(gr) => grs.push(gr),
            None => (),
        }

        Some(Box::new(Composite { generators: grs }))
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
            generators: vec![
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
