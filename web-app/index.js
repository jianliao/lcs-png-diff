import { generate_diff_png, dealloc, getWasm } from "lcs-png-diff";

(async () => {
  // const before = await (await fetch('/images/slider_before.png')).arrayBuffer();
  // const after = await (await fetch('/images/slider_after.png')).arrayBuffer();
  // const before_data = new Uint8Array(before);
  // const before_w = 1080;
  // const before_h = 14860;
  // const after_data = new Uint8Array(after);
  // const after_w = 1080;
  // const after_h = 14910;
  const button = document.getElementById('test');
  button.addEventListener('click', async () => {
    let canvas = document.createElement('canvas');
    const img_before = document.getElementById('before');
    canvas.width = img_before.width;
    canvas.height = img_before.height;
    let ctx = canvas.getContext('2d');
    ctx.drawImage(img_before, 0, 0);
    const before_data = ctx.getImageData(0, 0, ctx.canvas.width, ctx.canvas.height).data;

    canvas = document.createElement('canvas');
    const img_after = document.getElementById('after');
    canvas.width = img_after.width;
    canvas.height = img_after.height;
    ctx = canvas.getContext('2d');
    ctx.drawImage(img_after, 0, 0);
    const after_data = ctx.getImageData(0, 0, ctx.canvas.width, ctx.canvas.height).data;

    console.time('codezup');
    const result = generate_diff_png(before_data, img_before.width, /*img_before.height,*/ after_data, img_after.width, /*img_after.height*/);
    console.timeEnd('codezup');

    console.log(`Data ptr: ${result.data_ptr()}, Width: ${result.width()}, Length: ${result.length()}`);
    const data = new Uint8ClampedArray(getWasm().memory.buffer, result.data_ptr(), result.length());
    console.log(`Data length: ${data.length}`);

    canvas = document.createElement('canvas');
    canvas.width = result.width();
    canvas.height = result.height();
    const context = canvas.getContext('2d');
    const imgData = context.createImageData(result.width(), result.height());
    for (let i = data.length - 1; i >= 0; i--) {
      imgData.data[i] = data[i];
    }
    context.putImageData(imgData, 0, 0);
    document.getElementById('result').src = context.canvas.toDataURL('image/png');

    dealloc(result.data_ptr(), result.length());
    result.free();
  });
})();
