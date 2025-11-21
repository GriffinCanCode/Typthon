//! Tests for builtin functions

use super::*;
use super::iter::{typthon_range, typthon_range_next};
use super::print::{typthon_print_int, typthon_print_float, typthon_print_str};
use super::len::typthon_len;

#[test]
fn test_range_basic() {
    let mut r = range(0, 5, 1);
    assert_eq!(r.next(), Some(0));
    assert_eq!(r.next(), Some(1));
    assert_eq!(r.next(), Some(2));
    assert_eq!(r.next(), Some(3));
    assert_eq!(r.next(), Some(4));
    assert_eq!(r.next(), None);
}

#[test]
fn test_range_step() {
    let r = range(0, 10, 2);
    assert_eq!(r.collect::<Vec<_>>(), vec![0, 2, 4, 6, 8]);
}

#[test]
fn test_range_negative_step() {
    let mut r = range(10, 0, -2);
    assert_eq!(r.next(), Some(10));
    assert_eq!(r.next(), Some(8));
    assert_eq!(r.next(), Some(6));
    assert_eq!(r.next(), Some(4));
    assert_eq!(r.next(), Some(2));
    assert_eq!(r.next(), None);
}

#[test]
fn test_range_empty() {
    let mut r = range(0, 0, 1);
    assert_eq!(r.len(), 0);
    assert_eq!(r.next(), None);
}

#[test]
fn test_range_len() {
    assert_eq!(range(0, 10, 1).len(), 10);
    assert_eq!(range(0, 10, 2).len(), 5);
    assert_eq!(range(10, 0, -1).len(), 10);
}

#[test]
fn test_range_double_ended() {
    let mut r = range(0, 5, 1);
    assert_eq!(r.next(), Some(0));
    assert_eq!(r.next_back(), Some(4));
    assert_eq!(r.next(), Some(1));
    assert_eq!(r.next_back(), Some(3));
    assert_eq!(r.next(), Some(2));
    assert_eq!(r.next(), None);
    assert_eq!(r.next_back(), None);
}

#[test]
fn test_range_ffi() {
    let mut r = typthon_range(0, 5, 1);
    assert_eq!(typthon_range_next(&mut r), 0);
    assert_eq!(typthon_range_next(&mut r), 1);
    assert_eq!(typthon_range_next(&mut r), 2);
    assert_eq!(typthon_range_next(&mut r), 3);
    assert_eq!(typthon_range_next(&mut r), 4);
    assert_eq!(typthon_range_next(&mut r), i64::MIN);
}

#[test]
fn test_len_slice() {
    let arr = [1, 2, 3, 4, 5];
    assert_eq!(len(&arr[..]), 5);
    assert_eq!(len(&arr[..3]), 3);
}

#[test]
fn test_len_str() {
    assert_eq!(len("hello"), 5);
    assert_eq!(len(""), 0);
}

#[test]
fn test_print_ffi() {
    // Basic smoke test - just ensure no panics
    typthon_print_int(42);
    typthon_print_float(3.14);

    let s = "test";
    typthon_print_str(s.as_ptr(), s.len());

    typthon_print_str(core::ptr::null(), 0);
}

#[test]
fn test_len_ffi() {
    // Note: This test will fail until object header layout is implemented
    // For now, just test null safety
    assert_eq!(typthon_len(core::ptr::null()), 0);
}

