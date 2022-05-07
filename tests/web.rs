//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate lcs_png_diff;
extern crate wasm_bindgen_test;

use lcs_png_diff::*;
use std::ptr;
use wasm_bindgen::__rt::core::ffi::c_void;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
fn function_signature() {
    let result = generate_diff_png(
        String::from("3").as_mut_ptr(),
        1,
        2,
        3,
        String::from("3").as_mut_ptr(),
        4,
        5,
        6,
    );
    assert_eq!(
        result.len,
        12
    );
}
