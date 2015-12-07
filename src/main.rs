extern crate byteorder;

use std::io::Cursor;
use std::str;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};
use std::cmp::{Ordering, PartialOrd};

enum SimpleTypeDef {
    Int,
    Float,
    String,
}

struct SimpleType {
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
    println!("Hello, world!");

}
