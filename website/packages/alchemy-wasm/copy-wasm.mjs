import { copyFileSync, mkdirSync, statSync } from 'node:fs';
import { join } from 'node:path';

const source = join('dist', 'target-wasm', 'wasm32-unknown-unknown', 'release', 'alchemy_terminal_wasm.wasm');
const destinationDir = join('dist', 'packages', 'alchemy-wasm');
const destination = join(destinationDir, 'alchemy_terminal_wasm.wasm');

mkdirSync(destinationDir, { recursive: true });
copyFileSync(source, destination);
const size = statSync(destination).size;
if (size <= 0 || size > 2 * 1024 * 1024) {
  throw new Error(`Unexpected WASM size: ${size} bytes`);
}
console.log(`Copied ${destination} (${size} bytes).`);
