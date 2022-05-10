const { generate_diff_png, __wasm: wasm, dealloc } = require('lcs-png-diff');
const Jimp = require('jimp');
const { resolve } = require('path');

(async () => {
  const fixtureName = 'slider';
  const before = await Jimp.read(resolve(`tests/fixtures/${fixtureName}_before.png`));
  const after = await Jimp.read(resolve(`tests/fixtures/${fixtureName}_after.png`));

  console.time('codezup');
  const result = generate_diff_png(
    before.bitmap.data,
    before.bitmap.width,
    // before.bitmap.height,
    after.bitmap.data,
    after.bitmap.width,
    // after.bitmap.height
  );
  console.timeEnd('codezup');
})();