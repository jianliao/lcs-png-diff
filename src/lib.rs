extern crate base64;
extern crate lcs_diff;

mod utils;

use base64::{decode, encode};
use image::DynamicImage::ImageRgba8;
use image::*;
use lcs_diff::diff;
use lcs_diff::DiffResult::{Added, Common, Removed};
use std::cmp;
use wasm_bindgen::prelude::*;

pub static BLACK: (u8, u8, u8) = (0, 0, 0);
pub static RED: (u8, u8, u8) = (255, 119, 119);
pub static GREEN: (u8, u8, u8) = (99, 195, 99);
static RATE: f32 = 100.0 / 256.0;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[no_mangle]
pub fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        std::mem::drop(Vec::from_raw_parts(ptr, size, size));
    }
}

#[wasm_bindgen]
pub struct DiffResult {
    data_ptr: *const u8,
    length: u32,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl DiffResult {
    #[wasm_bindgen(constructor)]
    pub fn new(data_ptr: *mut u8, length: u32, width: u32, height: u32) -> DiffResult {
        DiffResult {
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
    before_h: usize,
    after: Vec<u8>,
    after_w: usize,
    after_h: usize,
) -> DiffResult {
    // Transform the flat byte array to two dimention grid and base64 encode each pixel row
    let before_bitmaps: Vec<String> = before
        .chunks(before_w * 4)
        .map(|chunk| encode(chunk))
        .collect();
    let after_bitmaps: Vec<String> = after
        .chunks(after_w * 4)
        .map(|chunk| encode(chunk))
        .collect();

    let diff_result = diff(&before_bitmaps, &after_bitmaps);

    let height = diff_result.len() as u32;
    let width = cmp::max(before_w, after_w) as u32;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for (y, d) in diff_result.iter().enumerate() {
        match d {
            &Added(ref a) => put_diff_pixels(y, &mut img, after_w as u32, &a.data, GREEN, RATE),
            &Removed(ref r) => put_diff_pixels(y, &mut img, before_w as u32, &r.data, RED, RATE),
            &Common(ref c) => put_diff_pixels(y, &mut img, width, &c.data, BLACK, 0.0),
        }
    }

    let image_raw_bytes = ImageRgba8(img).into_bytes();
    let length = image_raw_bytes.len() as u32;
    let data_ptr = image_raw_bytes.as_ptr();
    std::mem::forget(image_raw_bytes);
    DiffResult {
        data_ptr,
        length,
        width,
        height,
    }
}

fn blend(base: Rgba<u8>, rgb: (u8, u8, u8), rate: f32) -> Rgba<u8> {
    return Rgba([
        (base[0] as f32 * (1.0 - rate) + rgb.0 as f32 * (rate)) as u8,
        (base[1] as f32 * (1.0 - rate) + rgb.1 as f32 * (rate)) as u8,
        (base[2] as f32 * (1.0 - rate) + rgb.2 as f32 * (rate)) as u8,
        base[3],
    ]);
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
        before.dimensions().1 as usize,
        after.as_bytes().to_vec(),
        after.dimensions().0 as usize,
        after.dimensions().1 as usize,
    );
    println!(
        "Result width: {:?}, height: {:?}",
        result.width, result.height
    );
}
