#![allow(dead_code)]

pub fn eq<T: PartialEq>(a: T, b: T) -> bool {
    a == b
}

pub fn ne<T: PartialEq>(a: T, b: T) -> bool {
    a != b
}

pub fn deny<T: PartialEq>(_a: T, _b: T) -> bool {
    false
}
