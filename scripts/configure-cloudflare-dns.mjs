import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';

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

function wranglerToken() {
  const output = execFileSync('npx', ['wrangler@latest', 'auth', 'token'], { encoding: 'utf8' });
  const lines = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    if (/^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(lines[index])) return lines[index];
  }
  throw new Error('Could not read the Wrangler OAuth token. Set CLOUDFLARE_API_TOKEN in .env.');
}

const token = process.env.CLOUDFLARE_API_TOKEN || process.env.CF_API_TOKEN || process.env.CF_OAUTH_TOKEN || wranglerToken();
const zoneName = process.env.CLOUDFLARE_ZONE_NAME || 'tui-alchemy.sh';
const pagesHost = process.env.CLOUDFLARE_PAGES_HOST || 'tui-alchemy.pages.dev';
const apiBase = 'https://api.cloudflare.com/client/v4';

async function cloudflare(path, init = {}) {
  const response = await fetch(`${apiBase}${path}`, {
    ...init,
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
      ...(init.headers || {}),
    },
  });
  const text = await response.text();
  const body = text ? JSON.parse(text) : {};
  if (!response.ok || body.success === false) {
    const errors = Array.isArray(body.errors) && body.errors.length > 0
      ? body.errors.map((error) => `${error.code}: ${error.message}`).join('; ')
      : `${response.status} ${response.statusText}`;
    const error = new Error(errors);
    error.status = response.status;
    error.body = body;
    throw error;
  }
  return body.result;
}

async function zoneIdForName(name) {
  const query = new URLSearchParams({ name, status: 'active' });
  const zones = await cloudflare(`/zones?${query}`);
  if (!Array.isArray(zones) || zones.length !== 1) {
    throw new Error(`Expected exactly one active Cloudflare zone for ${name}, found ${zones?.length ?? 0}.`);
  }
  return zones[0].id;
}

async function recordsForName(zoneId, name) {
  const query = new URLSearchParams({ name, per_page: '100' });
  return cloudflare(`/zones/${zoneId}/dns_records?${query}`);
}

async function deleteRecord(zoneId, record) {
  await cloudflare(`/zones/${zoneId}/dns_records/${record.id}`, { method: 'DELETE' });
  console.log(`Deleted conflicting ${record.type} record for ${record.name}.`);
}

async function upsertCname(zoneId, name, content) {
  const existing = await recordsForName(zoneId, name);
  for (const record of existing) {
    if (record.type !== 'CNAME') {
      await deleteRecord(zoneId, record);
    }
  }

  const cname = existing.find((record) => record.type === 'CNAME');
  const payload = {
    type: 'CNAME',
    name,
    content,
    proxied: true,
    ttl: 1,
  };
  if (cname) {
    const updated = await cloudflare(`/zones/${zoneId}/dns_records/${cname.id}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    });
    console.log(`Updated CNAME ${updated.name} -> ${updated.content}.`);
    return;
  }

  const created = await cloudflare(`/zones/${zoneId}/dns_records`, {
    method: 'POST',
    body: JSON.stringify(payload),
  });
  console.log(`Created CNAME ${created.name} -> ${created.content}.`);
}

const zoneId = await zoneIdForName(zoneName);
await upsertCname(zoneId, zoneName, pagesHost);
await upsertCname(zoneId, `www.${zoneName}`, pagesHost);
console.log(`Configured ${zoneName} and www.${zoneName} DNS for Cloudflare Pages.`);
