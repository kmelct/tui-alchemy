import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { basename, join } from 'node:path';

if (existsSync('.env')) {
  for (const line of readFileSync('.env', 'utf8').split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const separator = trimmed.indexOf('=');
    if (separator <= 0) continue;
    const key = trimmed.slice(0, separator).trim();
    let value = trimmed.slice(separator + 1).trim();
    if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
      value = value.slice(1, -1);
    }
    if (!process.env[key]) process.env[key] = value;
  }
}

function packageVersion() {
  for (const line of readFileSync('Cargo.toml', 'utf8').split(/\r?\n/)) {
    const match = /^version = "([^"]+)"$/.exec(line);
    if (match) return match[1];
  }
  throw new Error('Cargo.toml package version not found.');
}

function wranglerToken() {
  const output = execFileSync('npx', ['wrangler@latest', 'auth', 'token'], { encoding: 'utf8' });
  const lines = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    if (/^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(lines[index])) return lines[index];
  }
  throw new Error('Could not read the Wrangler OAuth token. Set CLOUDFLARE_API_TOKEN in .env.');
}

const token = process.env.CLOUDFLARE_API_TOKEN || process.env.CF_API_TOKEN || process.env.CF_OAUTH_TOKEN || wranglerToken();
const accountId = process.env.CLOUDFLARE_R2_ACCOUNT_ID || '279a8319536bf8f797e9d25954fe445c';
const bucket = process.env.CLOUDFLARE_R2_BUCKET || 'tui-alchemy-assets';
const version = packageVersion();
const apiBase = `https://api.cloudflare.com/client/v4/accounts/${accountId}/r2/buckets/${bucket}/objects`;

const uploads = [
  {
    key: 'i.tui-alchemy.sh',
    file: 'website/dist/i.tui-alchemy.sh',
    contentType: 'text/x-shellscript; charset=utf-8',
    cacheControl: 'public, max-age=300',
  },
  {
    key: 'install.ps1',
    file: 'website/dist/install.ps1',
    contentType: 'text/plain; charset=utf-8',
    cacheControl: 'public, max-age=300',
  },
  {
    key: `downloads/tui-alchemy-${version}.crate`,
    file: `website/dist/downloads/tui-alchemy-${version}.crate`,
    contentType: 'application/gzip',
    cacheControl: 'public, max-age=31536000, immutable',
  },
];

const downloadsDir = join('website', 'dist', 'downloads');
if (existsSync(downloadsDir)) {
  for (const name of readdirSync(downloadsDir)) {
    if (!name.startsWith(`tui-alchemy-${version}-`) || !name.endsWith('.tar.gz')) continue;
    uploads.push({
      key: `downloads/${name}`,
      file: join(downloadsDir, name),
      contentType: 'application/gzip',
      cacheControl: 'public, max-age=31536000, immutable',
    });
  }
}

function objectUrl(key) {
  return `${apiBase}/${key.split('/').map(encodeURIComponent).join('/')}`;
}

for (const upload of uploads) {
  const body = readFileSync(upload.file);
  const response = await fetch(objectUrl(upload.key), {
    method: 'PUT',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': upload.contentType,
      'Cache-Control': upload.cacheControl,
    },
    body,
  });
  const text = await response.text();
  const payload = text ? JSON.parse(text) : {};
  if (!response.ok || payload.success === false) {
    const errors = Array.isArray(payload.errors) && payload.errors.length > 0
      ? payload.errors.map((error) => `${error.code}: ${error.message}`).join('; ')
      : `${response.status} ${response.statusText}`;
    throw new Error(`Failed to upload ${upload.key}: ${errors}`);
  }
  console.log(`Uploaded ${upload.key} from ${basename(upload.file)}.`);
}
