extern crate base64;

mod utils;

use base64::{decode, encode};
use image::DynamicImage::ImageRgba8;
use image::*;
use std::cmp;
use wasm_bindgen::prelude::*;

pub static BLACK: (u8, u8, u8) = (0, 0, 0);
pub static RED: (u8, u8, u8) = (255, 119, 119);
pub static GREEN: (u8, u8, u8) = (99, 195, 99);
static RATE: f32 = 0.25;

#[derive(Debug, PartialEq)]
enum DiffResult<'a, T: PartialEq> {
    Removed(DiffElement<'a, T>),
    Common(DiffElement<'a, T>),
    Added(DiffElement<'a, T>),
}

#[derive(Debug, PartialEq)]
struct DiffElement<'a, T: PartialEq> {
    pub old_index: Option<usize>,
    pub new_index: Option<usize>,
    pub data: &'a T,
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern "C" {
//     // Use `js_namespace` here to bind `console.log(..)` instead of just
//     // `log(..)`
//     #[wasm_bindgen(js_namespace = console)]
//     fn log(s: &str);
// }

// macro_rules! console_log {
//     ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
// }

#[wasm_bindgen]
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        std::mem::drop(Vec::from_raw_parts(ptr, size, size));
    }
}

#[wasm_bindgen]
pub struct PngDiffResult {
    data_ptr: *const u8,
    length: u32,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl PngDiffResult {
    #[wasm_bindgen(constructor)]
    pub fn new(data_ptr: *mut u8, length: u32, width: u32, height: u32) -> PngDiffResult {
        PngDiffResult {
            data_ptr,
            length,
            width,
            height,
        }
    }

    pub fn data_ptr(&self) -> *const u8 {
        self.data_ptr
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

#[wasm_bindgen]
pub fn generate_diff_png(
    before: Vec<u8>,
    before_w: usize,
    // before_h: usize,
    after: Vec<u8>,
    after_w: usize,
    // after_h: usize,
) -> PngDiffResult {
    // Transform the flat byte array to two dimention grid and base64 encode each pixel row
    let before_bitmaps: Vec<String> = before
        .chunks(before_w * 4)
        .map(encode)
        .collect();
    let after_bitmaps: Vec<String> = after
        .chunks(after_w * 4)
        .map(encode)
        .collect();

    let diff_result = lcs_diff(&before_bitmaps, &after_bitmaps);

    let height = diff_result.len() as u32;
    let width = cmp::max(before_w, after_w) as u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for (y, d) in diff_result.iter().enumerate() {
        match *d {
            DiffResult::Added(ref a) => {
                put_diff_pixels(y, &mut img, after_w as u32, a.data, GREEN, RATE)
            }
            DiffResult::Removed(ref r) => {
                put_diff_pixels(y, &mut img, before_w as u32, r.data, RED, RATE)
            }
            DiffResult::Common(ref c) => put_diff_pixels(y, &mut img, width, c.data, BLACK, 0.0),
        }
    }

    let image_raw_bytes = ImageRgba8(img).into_bytes();
    let length = image_raw_bytes.len() as u32;
    let data_ptr = image_raw_bytes.as_ptr();
    std::mem::forget(image_raw_bytes);
    PngDiffResult {
        data_ptr,
        length,
        width,
        height,
    }
}

fn blend(base: Rgba<u8>, rgb: (u8, u8, u8), rate: f32) -> Rgba<u8> {
    Rgba([
        (base[0] as f32 * (1.0 - rate) + rgb.0 as f32 * (rate)) as u8,
        (base[1] as f32 * (1.0 - rate) + rgb.1 as f32 * (rate)) as u8,
        (base[2] as f32 * (1.0 - rate) + rgb.2 as f32 * (rate)) as u8,
        base[3],
    ])
}

fn put_diff_pixels(
    y: usize,
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    row_width: u32,
    data: &String,
    rgb: (u8, u8, u8),
    rate: f32,
) {
    let row = decode(data).expect("Unable base64 decode");
    for x in 0..img.dimensions().0 {
        let index = x as usize * 4;
        let pixel: Rgba<u8> = if row_width > x {
            Rgba([row[index], row[index + 1], row[index + 2], row[index + 3]])
        } else {
            Rgba([0, 0, 0, 0])
        };
        img.put_pixel(x as u32, y as u32, blend(pixel, rgb, rate));
    }
}

fn create_table<T: PartialEq>(old: &[T], new: &[T]) -> Vec<Vec<u32>> {
    let new_len = new.len();
    let old_len = old.len();
    let mut table = vec![vec![0; old_len + 1]; new_len + 1];
    for i in 0..new_len {
        let i = new_len - i - 1;
        table[i][old_len] = 0;
        for j in 0..old_len {
            let j = old_len - j - 1;
            table[i][j] = if new[i] == old[j] {
                table[i + 1][j + 1] + 1
            } else {
                cmp::max(table[i + 1][j], table[i][j + 1])
            }
        }
    }
    table
}

fn lcs_diff<'a, T: PartialEq>(old: &'a [T], new: &'a [T]) -> Vec<DiffResult<'a, T>> {
    let mut result: Vec<DiffResult<T>> = Vec::new();
    let new_len = new.len();
    let old_len = old.len();

    if new_len == 0 {
        let mut o = 0;
        while o < old_len {
            result.push(DiffResult::Removed(DiffElement {
                old_index: Some(o),
                new_index: None,
                data: &old[o],
            }));
            o += 1;
        }
        return result;
    } else if old_len == 0 {
        let mut n = 0;
        while n < new_len {
            result.push(DiffResult::Added(DiffElement {
                old_index: None,
                new_index: Some(n),
                data: &new[n],
            }));
            n += 1;
        }
        return result;
    } else {
        let mut o = 0;
        let mut n = 0;
        let common_prefix = old.iter().zip(new).take_while(|p| p.0 == p.1);
        let prefix_size = common_prefix.count();
        let common_suffix = old
            .iter()
            .rev()
            .zip(new.iter().rev())
            .take(cmp::min(old_len, new_len) - prefix_size)
            .take_while(|p| p.0 == p.1);
        let suffix_size = common_suffix.count();
        let table = create_table(
            &old[prefix_size..(old_len - suffix_size)],
            &new[prefix_size..(new_len - suffix_size)],
        );
        let new_len = new_len - prefix_size - suffix_size;
        let old_len = old_len - prefix_size - suffix_size;

        // Restore common prefix
        let mut prefix_index = 0;
        while prefix_index < prefix_size {
            result.push(DiffResult::Common(DiffElement {
                old_index: Some(prefix_index),
                new_index: Some(prefix_index),
                data: &old[prefix_index],
            }));
            prefix_index += 1;
        }

        loop {
            if n >= new_len || o >= old_len {
                break;
            }
            let new_index = n + prefix_size;
            let old_index = o + prefix_size;
            if new[new_index] == old[old_index] {
                result.push(DiffResult::Common(DiffElement {
                    old_index: Some(old_index),
                    new_index: Some(new_index),
                    data: &new[new_index],
                }));
                n += 1;
                o += 1;
            } else if table[n + 1][o] >= table[n][o + 1] {
                result.push(DiffResult::Added(DiffElement {
                    old_index: None,
                    new_index: Some(new_index),
                    data: &new[new_index],
                }));
                n += 1;
            } else {
                result.push(DiffResult::Removed(DiffElement {
                    old_index: Some(old_index),
                    new_index: None,
                    data: &old[old_index],
                }));
                o += 1;
            }
        }
        while n < new_len {
            let new_index = n + prefix_size;
            result.push(DiffResult::Added(DiffElement {
                old_index: None,
                new_index: Some(new_index),
                data: &new[new_index],
            }));
            n += 1;
        }
        while o < old_len {
            let old_index = o + prefix_size;
            result.push(DiffResult::Removed(DiffElement {
                old_index: Some(old_index),
                new_index: None,
                data: &old[old_index],
            }));
            o += 1;
        }

        // Restore common suffix
        let mut suffix_index = 0;
        while suffix_index < suffix_size {
            let old_index = suffix_index + old_len + prefix_size;
            let new_index = suffix_index + new_len + prefix_size;
            result.push(DiffResult::Common(DiffElement {
                old_index: Some(old_index),
                new_index: Some(new_index),
                data: &old[old_index],
            }));
            suffix_index += 1;
        }
    }
    result
}

#[test]
fn exploration() {
    let before =
        image::open("/Users/jianliao/Work/scm/github/lcs-png-diff/tests/fixtures/reference.png")
            .unwrap();
    let after = image::open("/Users/jianliao/Work/scm/github/lcs-png-diff/tests/fixtures/test.png")
        .unwrap();

    let result = generate_diff_png(
        before.as_bytes().to_vec(),
        before.dimensions().0 as usize,
        // before.dimensions().1 as usize,
        after.as_bytes().to_vec(),
        after.dimensions().0 as usize,
        // after.dimensions().1 as usize,
    );
    println!(
        "Result width: {:?}, height: {:?}",
        result.width, result.height
    );
}
