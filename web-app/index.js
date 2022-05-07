import { generate_diff_png } from "lcs-png-diff";

const result = generate_diff_png([1, 2, 3], 1, 1, [3, 2, 1], 2, 2);

console.log(result);
