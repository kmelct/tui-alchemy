import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function packageVersion() {
  const manifest = execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], { encoding: 'utf8' });
  return JSON.parse(manifest).packages.find((pkg) => pkg.name === 'tui-alchemy')?.version;
}

function hostTriple() {
  const verbose = execFileSync('rustc', ['-vV'], { encoding: 'utf8' });
  const line = verbose.split(/\r?\n/).find((entry) => entry.startsWith('host: '));
  if (!line) throw new Error('rustc -vV did not report a host triple.');
  return line.slice('host: '.length).trim();
}

const version = packageVersion();
if (!version) throw new Error('Could not determine tui-alchemy package version.');
const triple = hostTriple();
const exeSuffix = triple.includes('windows') ? '.exe' : '';
const binaryPath = join('target', 'release', `tui-alchemy${exeSuffix}`);

execFileSync('cargo', ['build', '--release', '--locked'], { stdio: 'inherit' });
if (!existsSync(binaryPath)) throw new Error(`Expected release binary at ${binaryPath}.`);

mkdirSync(join('website', 'dist', 'downloads'), { recursive: true });
const staging = join(tmpdir(), `tui-alchemy-${process.pid}-${Date.now()}`);
mkdirSync(staging, { recursive: true });
try {
  copyFileSync(binaryPath, join(staging, `tui-alchemy${exeSuffix}`));
  const archive = join('website', 'dist', 'downloads', `tui-alchemy-${version}-${triple}.tar.gz`);
  execFileSync('tar', ['-czf', archive, '-C', staging, `tui-alchemy${exeSuffix}`], { stdio: 'inherit' });
  console.log(`Packaged ${archive}`);
} finally {
  rmSync(staging, { recursive: true, force: true });
}
