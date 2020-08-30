use std::io::{Result, Write};

use rand::Rng;
use uuid::Uuid;
use regex::Regex;

trait Generust {
    fn generate(&self, w: &mut dyn Write) -> Result<()>;
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

struct Timestamp {
}

impl Generust for Timestamp {
    fn generate(&self, w: &mut dyn Write) -> Result<()> {
        write!(w, "{}", chrono::Utc::now().format("%+"))
    }
}

fn parse(text: &str) -> Result<Composite> {
    let mut grs: Vec<Box<dyn Generust>> = vec![];
    let mut start = 0;
    let rx = Regex::new(r"(\$\{([A-Z]+)})").unwrap();
    for cap in rx.captures_iter(text) {

        let outer = cap.get(1).unwrap();
        let inner = cap.get(2).unwrap();
        println!("outer: {}..{}, {}", outer.start(), outer.end(), outer.as_str());
        println!("inner: {}..{}, {}", inner.start(), inner.end(), inner.as_str());

        // Text block if any
        if outer.start() > start {
            let txt = String::from(&text[start .. outer.start()]);
            println!("txt: '{}'", txt);
            grs.push(Box::new(Text{ text: txt }))
        }

        // Generust
        if inner.as_str().eq("UUID") {
            grs.push(Box::new( Uuid4{} ));
        } else if inner.as_str().eq("TIMESTAMP") {
            grs.push(Box::new( Timestamp{} ));
        }

        start = outer.end();
    }

    if start < text.len() {
        let txt = String::from(&text[start .. ]);
        println!("txt: '{}'", txt);
        grs.push(Box::new(Text{ text: txt }))
    }

    Ok(Composite { generators: grs })
}

#[cfg(test)]
mod test {
    use crate::{Uuid4, Generust, Integer, Timestamp, Text, Composite, parse};
    use uuid::Uuid;
    use chrono::DateTime;

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
        let gr = Uuid4 {};
        assert!(gr.generate(&mut buf).is_ok());
        let str = str(buf);
        assert!(Uuid::parse_str(&str).is_ok());
        println!("uuid4: {}", str);
    }

    #[test]
    fn test_integer() {
        let mut buf = buf(16);
        let gr = Integer {min: 0, max: 100};
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
        let gr= parse("{ uuid: ${UUID}, timestamp: ${TIMESTAMP} }").unwrap();
        for _ in 0..100 {
            gr.generate(& mut buf);
        }
        println!("{}", str(buf));
    }
}
