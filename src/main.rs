#![feature(plugin)]
#![plugin(peg_syntax_ext)]

extern crate byteorder;
extern crate bufstream;

use std::io::Cursor;
use std::str;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::thread;
use bufstream::BufStream;

use std::io::{Read, BufRead, Write};

use std::sync::{Mutex, Arc};

type DB = Arc<Mutex<Database>>;

#[derive(Debug)]
pub enum SimpleTypeDef {
    Int,
    Float,
    String,
}

impl fmt::Display for SimpleTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let otype = match self {
            &SimpleTypeDef::Int => "int",
            &SimpleTypeDef::Float => "float",
            &SimpleTypeDef::String => "string"
        };
        write!(f, "{}", otype)
    }
}

impl PartialEq for SimpleTypeDef {
    fn eq(&self, other: &SimpleTypeDef) -> bool {
        match (self, other) {
            (&SimpleTypeDef::Int, &SimpleTypeDef::Int) => true,
            (&SimpleTypeDef::Float, &SimpleTypeDef::Float) => true,
            (&SimpleTypeDef::String, &SimpleTypeDef::String) => true,
            _ => false
        }
    }
}

#[derive(Debug)]
pub struct SimpleType {
    otype: SimpleTypeDef,
    value: Vec<u8>,
}

impl fmt::Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let val = match self.otype {
            SimpleTypeDef::Int =>
                Cursor::new(&self.value).read_i64::<BigEndian>().unwrap().to_string(),
            SimpleTypeDef::Float =>
                Cursor::new(&self.value).read_f64::<BigEndian>().unwrap().to_string(),
            SimpleTypeDef::String =>
                str::from_utf8(&self.value).unwrap().to_string()
        };
        write!(f, "{}:{}", self.otype, val)
    }

}

enum SimpleTypeError {
    NonMatchingType
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

#[derive(Debug)]
struct Database {
    v: Option<SimpleType>,
    t: Option<SimpleTypeDef>,
}
#[derive(Debug)]
pub enum Operation {
    Gt, Lt, Gte, Lte, Eq
}

#[derive(Debug)]
pub enum DatabaseError {
    TypeError
}

impl Database {
    fn new() -> Database {
        Database{v:None, t:None}
    }
    fn set(&mut self, v: SimpleType) -> Result<(), DatabaseError> {
        if self.t.is_none() {
            return Err(DatabaseError::TypeError);
        }
        if let Some(ref existing_type) = self.t {
            // if val
            if *existing_type != v.otype {
                return Err(DatabaseError::TypeError);
            }
        }
        println!("Setting db to {:?}", v);
        self.v = Some(v);
        Ok(())
    }

    fn get(&self) -> &Option<SimpleType> {
        &self.v
    }

    fn compare(&mut self, comparison: SimpleType, operation: Operation) -> bool {
        if let Some(ref dbval) = self.v {
            println!("safely comparing: {:?}", comparison);
            let result = match operation {
                Operation::Gt =>
                    dbval > &comparison,
                Operation::Gte =>
                    dbval >= &comparison,
                Operation::Lte =>
                    dbval <= &comparison,
                Operation::Lt =>
                    dbval < &comparison,
                Operation::Eq =>
                    dbval == &comparison,
            };
            return result;

        };
        false

    }
}

#[test]
fn test_db_compare() {
    let mut db = Database::new();
    let four = SimpleType::from_int(4);
    let six = SimpleType::from_int(6);

    assert!(db.set(SimpleType::from_int(5)).is_err());

    db.t = Some(SimpleTypeDef::Int);
    db.set(SimpleType::from_int(5));

    assert!(db.compare(four, Operation::Gte));
}

#[test]
fn test_set_type() {
    let mut db = Database::new();
    // WTF?
    db.t = Some(SimpleTypeDef::Int);
    let s = SimpleType::from_int(1);
    db.set(s).expect("Error setting type");

    assert!(db.set(SimpleType::from_float(1.0)).is_err(), "was expecting a type error");

}


fn main() {
    println!("Starting up the worst database of all time.  What a giant mistake you have made.");
    println!("Starting on port 6000");

    let db = Arc::new(Mutex::new(Database::new()));

    let tcp = TcpListener::bind("127.0.0.1:6000").unwrap();
    for stream in tcp.incoming() {

        if let Ok(s) = stream {
            println!("Someone has made the terrible decision of connecting.");
            let db2 = db.clone();
            thread::spawn(move || handle_client(s, db2) );
        } else {
            println!("Massive failure");
        }
    }

    println!("Goodbye, you will regret ever having run this.");
}

fn handle_client(mut stream: TcpStream, mut db: DB ) {
    let buffer = BufStream::new(stream.try_clone().unwrap());

    for line in buffer.lines() {
        match line {
            Ok(l) => {
                println!("Got something {}", l);
                if let Ok(p) = statement(&l) {
                    println!("{:?}", p);
                    handle_command(&stream, p, &db);
                } else {
                    println!("Parse error");
                    stream.write("parse_error".as_bytes());
                    continue;
                }
            },
            Err(_) => {
                println!("Geee, an error.  Shocker.  Not really though.  Worst DB ever.");
            }
        }
    }

}


fn handle_command(mut stream: &TcpStream, command: UselessStatement, mut db: &DB ) {
    println!("Acquiring mutex lock");
    let mut database = db.lock().unwrap();
    println!("Acquired mutex lock");
    match command {
        UselessStatement::SetType(def) => {
            println!("Resetting type");
            database.t = Some(def);
            stream.write("ok\n".as_bytes());
        },
        UselessStatement::SetVar(t) => { // SimpleType
            // does the type match?  need to check it here
            match database.set(t) {
                Ok(_) => { stream.write("ok\n".as_bytes()); },
                Err(_) => { stream.write("type_error\n".as_bytes()); }
            };
        },
        UselessStatement::Comparison(simple_type, op) => {
            println!("We have a comparison");
            let result = database.compare(simple_type, op);
            println!("Compare result: {}", result);
            let response = format!("{}\n", result);
            stream.write(response.as_bytes());
        },
        UselessStatement::Get => {
            println!("Getting var");
            if let &Some(ref tmp) = database.get() {
                println!("var = {:?}", tmp);
                stream.write(format!("{}\n", tmp).as_bytes());
            } else {
                stream.write("None\n".as_bytes());
            }
        }
        //_ => {},
    };
}

enum CommandError {
    ParseError
}

#[derive(Debug)]
pub enum UselessStatement {
    SetType(SimpleTypeDef),
    SetVar(SimpleType),
    Comparison(SimpleType, Operation),
    Get
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
var
*/

use useless::{variable,raw_string,escaped_quote, quoted_string, set_command, statement, comparison_command, comparison};

#[test]
fn test_variable_parsing() {
    let v = variable("4").unwrap();
    assert!(v == SimpleType::from_int(4));

    let v = variable("4.0").unwrap();
    assert!(v == SimpleType::from_float(4.0));

}

#[test]
fn test_explicit_quoted_string_parsing() {
    let s = r#""You are a monkey""#;
    let v = quoted_string(s).unwrap();
}

#[test]
fn test_string_parsing() {
    let s = r#""You are a monkey""#;
    println!("expecting quotes: {}", s);
    let v = variable(s).unwrap();
    assert!(v == SimpleType::from_string(s));
}

#[test]
fn test_simple_string() {
    raw_string("test").unwrap();
}

#[test]
fn test_escaped_quote() {
    escaped_quote(r#"\""#).unwrap();
}

#[test]
fn test_set_command() {
    set_command("var = 1").unwrap();
}

#[test]
fn test_compare_type() {
    assert!(SimpleTypeDef::Int == SimpleTypeDef::Int);
    assert!(SimpleTypeDef::Int != SimpleTypeDef::Float);
    assert!(SimpleTypeDef::Int != SimpleTypeDef::String);
}

#[test]
fn test_comparison_parsing() {
    comparison(">").expect(">");
    comparison("<").expect("<");
    comparison("<=").expect("<=");
    comparison(">=").expect(">=");

    comparison_command("var > 5").unwrap();
    comparison_command("var >= 5").unwrap();
    // comparison_command("var <= 5").unwrap();
    // comparison_command("var < 5").unwrap();
}
