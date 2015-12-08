#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate byteorder;
extern crate bufstream;

use std::io::Cursor;
use std::str;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::cmp::{Ordering, PartialOrd};

use std::net::{TcpListener, TcpStream};
use std::thread;
use bufstream::BufStream;

use std::io::{Read, BufRead, Write};


enum SimpleTypeDef {
    Int,
    Float,
    String,
}

pub struct SimpleType {
    otype: SimpleTypeDef,
    value: Vec<u8>,
}

impl SimpleType {
    fn from_int(val: i64) -> SimpleType {

        let mut v = Vec::new();
        v.write_i64::<BigEndian>(val).unwrap();
        SimpleType{ otype: SimpleTypeDef::Int,
                    value: v }
    }

    fn from_float(val: f64) -> SimpleType {
        let mut v = Vec::new();
        v.write_f64::<BigEndian>(val).unwrap();
        SimpleType{ otype: SimpleTypeDef::Float,
                    value: v}
    }

    fn from_string(val: &str) -> SimpleType {
        let mut v = val.bytes().collect();
        SimpleType{ otype: SimpleTypeDef::String,
                    value: v }
    }

}

impl PartialOrd for SimpleType {
    fn partial_cmp(&self, other: &SimpleType) -> Option<Ordering> {

        match  (&self.otype, &other.otype) {
            (&SimpleTypeDef::Int, &SimpleTypeDef::Int) => {
                // let (x, y) = (self.value.read_i64::<BigEndian>().unwrap(),
                //               other.value.read_i64::<BigEndian>().unwrap());
                let x = Cursor::new(&self.value).read_i64::<BigEndian>().unwrap();
                let y = Cursor::new(&other.value).read_i64::<BigEndian>().unwrap();
                return x.partial_cmp(&y);
                // Some(Ordering::Equal)
            },
            (&SimpleTypeDef::Float, &SimpleTypeDef::Float) => {
                let x = Cursor::new(&self.value).read_f64::<BigEndian>().unwrap();
                let y = Cursor::new(&other.value).read_f64::<BigEndian>().unwrap();
                return x.partial_cmp(&y);
            },
            (&SimpleTypeDef::String, &SimpleTypeDef::String) => {
                let x = str::from_utf8(&self.value).unwrap();
                let y = str::from_utf8(&other.value).unwrap();
                return x.partial_cmp(&y);
            },
            (_, _) => None
        }


    }
}
impl PartialEq for SimpleType {
    fn eq(&self, other: &SimpleType) -> bool {
        true
    }
}

#[test]
fn test_fail_compare_diff_types() {
    let t = SimpleType::from_int(1);
    let s = SimpleType::from_float(1.0);

    assert_eq!(t.partial_cmp(&s), None);

}

#[test]
fn test_int_comparisons() {
    let t = SimpleType::from_int(1);
    let k = SimpleType::from_int(2);
    let z = SimpleType::from_int(1);

    assert!(k > t);
    assert!(t < k);
    assert!(z == t);
}

#[test]
fn test_float_comparisons() {
    let t = SimpleType::from_float(1.0);
    let k = SimpleType::from_float(2.0);
    let z = SimpleType::from_float(1.0);

    assert!(k > t);
    assert!(t < k);
    assert!(z == t);
}

#[test]
fn test_string_comparisons() {
    let t = SimpleType::from_string("bacon");
    let k = SimpleType::from_string("eggs");
    let z = SimpleType::from_string("bacon");

    assert!(k > t);
    assert!(t < k);
    assert!(z == t);
}



fn main() {
    println!("Starting up the worst database of all time.  What a giant mistake you have made.");
    println!("Starting on port 6000");

    let tcp = TcpListener::bind("127.0.0.1:6000").unwrap();
    for stream in tcp.incoming() {
        if let Ok(s) = stream {
            println!("Someone has made the terrible decision of connecting.");
            thread::spawn(move || handle_client(s) );
        } else {
            println!("Massive failure");
        }
    }

    println!("Goodbye, you will regret ever having run this.");
}

fn handle_client(mut stream: TcpStream) {
    let buffer = BufStream::new(stream.try_clone().unwrap());

    for line in buffer.lines() {
        match line {
            Ok(l) => {
                println!("Got something {}", l);
                let result = handle_command(&l);
            },
            Err(_) => {
                println!("Geee, an error.  Shocker.  Not really though.  Worst DB ever.");
            }
        }
    }

}

fn handle_command(line: &str) {

}

enum UselessStatement {
    SetType(SimpleTypeDef),
    SetVar(SimpleType),
    Comparison(SimpleType)
}

peg_file! useless("useless.rustpeg");


/*
type int|float|string
var = 4.5
var = "this is a var"
var = 5
var > 10
var < 5.0
var == 2.0
get
*/

use useless::{variable,raw_string};

#[test]
fn test_variable_parsing() {
    let v = variable("4").unwrap();
    assert!(v == SimpleType::from_int(4));

    let v = variable("4.0").unwrap();
    assert!(v == SimpleType::from_float(4.0));

}

#[test]
fn test_string_parsing() {
    let s = r#"You are a monkey"#;
    let v = variable(s).unwrap();
    assert!(v == SimpleType::from_string(s));
}

#[test]
fn test_simple_string() {
    raw_string("test").unwrap();
}
