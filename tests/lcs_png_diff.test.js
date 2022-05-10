const { generate_diff_png, __wasm: wasm, dealloc } = require('lcs-png-diff');
const Jimp = require('jimp');
const { resolve } = require('path');

const prepareTestFixtures = async fixtureName => {
  const before = await Jimp.read(resolve(`tests/fixtures/${fixtureName}_before.png`));
  const after = await Jimp.read(resolve(`tests/fixtures/${fixtureName}_after.png`));
  return { before: before.bitmap, after: after.bitmap };
}

const validatePNG = async (fixtureName, { data, width, height }) => {
  const resultImg = await new Jimp({ data, width, height });
  await resultImg.writeAsync(resolve(`tests/fixtures/results/${fixtureName}.png`));
  const res = await Jimp.read(resolve(`tests/fixtures/results/${fixtureName}.png`));
  return { width: res.bitmap.width, height: res.bitmap.height }
}

// test('Backstop Price', async () => {
//   const { before, after } = await prepareTestFixtures('backstopjs_pricing');

//   const result = generate_diff_png(
//     before.data,
//     before.width,
//  // before.height,
//     after.data,
//     after.width,
//  // after.height
//   );

//   try {
//     const data = new Uint8Array(wasm.memory.buffer, result.data_ptr(), result.length());
//     const { width, height } = await validatePNG('backstopjs_pricing', { data, width: result.width(), height: result.height() });
//     expect(width).toBe(320);
//     expect(height).toBe(1739);
//   } finally {
//     dealloc(result.data_ptr(), result.length());
//     result.free();
//   }
// }, 50000);

test('Slider', async () => {
  const { before, after } = await prepareTestFixtures('slider');

  const result = generate_diff_png(
    before.data,
    before.width,
    // before.height,
    after.data,
    after.width,
    // after.height
  );

  try {
    const data = new Uint8Array(wasm.memory.buffer, result.data_ptr(), result.length());
    const { width, height } = await validatePNG('slider', { data, width: result.width(), height: result.height() });
    expect(width).toBe(320);
    expect(height).toBe(1739);
  } finally {
    dealloc(result.data_ptr(), result.length());
    result.free();
  }
}, 50000);

// test('Text area', async () => {
//   const { before, after } = await prepareTestFixtures('text-area');

//   const result = generate_diff_png(
//     before.data,
//     before.width,
//  // before.height,
//     after.data,
//     after.width,
//  // after.height
//   );

//   try {
//     const data = new Uint8Array(wasm.memory.buffer, result.data_ptr(), result.length());
//     const { width, height } = await validatePNG('text-area', { data, width: result.width(), height: result.height() });
//     expect(width).toBe(320);
//     expect(height).toBe(1739);
//   } finally {
//     dealloc(result.data_ptr(), result.length());
//     result.free();
//   }
// }, 50000);
