import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';

const dotenvPath = '.env';
if (existsSync(dotenvPath)) {
  for (const line of readFileSync(dotenvPath, 'utf8').split(/\r?\n/)) {
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

const accountId = process.env.CLOUDFLARE_PAGES_ACCOUNT_ID || process.env.CLOUDFLARE_ACCOUNT_ID || 'e9c375806f33a6c2a42c7d5ca9729105';
const projectName = process.env.CLOUDFLARE_PAGES_PROJECT || 'tui-alchemy';
const productionBranch = process.env.CLOUDFLARE_PAGES_BRANCH || 'master';
const githubOwner = process.env.CLOUDFLARE_GITHUB_OWNER || 'kmelct';
const githubRepo = process.env.CLOUDFLARE_GITHUB_REPO || 'tui-alchemy';
const githubOwnerId = process.env.CLOUDFLARE_GITHUB_OWNER_ID || '22073208';
const githubRepoId = process.env.CLOUDFLARE_GITHUB_REPO_ID || '1256537015';
const domainList = (process.env.CLOUDFLARE_PAGES_DOMAINS || 'tui-alchemy.sh,i.tui-alchemy.sh,www.tui-alchemy.sh')
  .split(',')
  .map((domain) => domain.trim())
  .filter(Boolean);

function wranglerToken() {
  const output = execFileSync('npx', ['wrangler@latest', 'auth', 'token'], { encoding: 'utf8' });
  const lines = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    if (/^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(lines[index])) return lines[index];
  }
  throw new Error('Could not read the Wrangler OAuth token. Set CLOUDFLARE_API_TOKEN in .env.');
}

const token = process.env.CLOUDFLARE_API_TOKEN || process.env.CF_API_TOKEN || process.env.CF_OAUTH_TOKEN || wranglerToken();
const apiBase = 'https://api.cloudflare.com/client/v4';
const projectPath = `/accounts/${accountId}/pages/projects/${encodeURIComponent(projectName)}`;
const buildConfig = {
  build_command: 'sh scripts/build-website.sh',
  destination_dir: 'website/dist',
  root_dir: '/',
  build_caching: true,
};
const gitSource = {
  type: 'github',
  config: {
    owner: githubOwner,
    owner_id: githubOwnerId,
    repo_name: githubRepo,
    repo_id: githubRepoId,
    production_branch: productionBranch,
    production_deployments_enabled: true,
    preview_deployment_setting: 'all',
    pr_comments_enabled: true,
  },
};

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

async function projectExists() {
  try {
    return await cloudflare(projectPath, { method: 'GET' });
  } catch (error) {
    if (error.status === 404) return null;
    throw error;
  }
}

async function createDirectUploadProject() {
  const created = await cloudflare(`/accounts/${accountId}/pages/projects`, {
    method: 'POST',
    body: JSON.stringify({
      name: projectName,
      production_branch: productionBranch,
      build_config: buildConfig,
    }),
  });
  console.log(`Created Direct Upload Pages project ${created.name}.`);
  return created;
}

async function ensureProject() {
  const current = await projectExists();
  if (current) {
    const updated = await cloudflare(projectPath, {
      method: 'PATCH',
      body: JSON.stringify({
        production_branch: productionBranch,
        build_config: buildConfig,
        deployment_configs: current.deployment_configs,
      }),
    });
    console.log(`Updated Pages project ${updated.name} build config.`);
    return updated;
  }

  try {
    const created = await cloudflare(`/accounts/${accountId}/pages/projects`, {
      method: 'POST',
      body: JSON.stringify({
        name: projectName,
        production_branch: productionBranch,
        build_config: buildConfig,
        source: gitSource,
      }),
    });
    console.log(`Created native Git Pages project ${created.name}.`);
    return created;
  } catch (error) {
    if (!String(error.message).includes('8000011')) throw error;
    console.warn('Cloudflare Pages Git installation returned 8000011; falling back to a Direct Upload project for CI/CD deployment.');
    return createDirectUploadProject();
  }
}

async function ensureZoneVisible() {
  const apex = 'tui-alchemy.sh';
  const query = new URLSearchParams({ name: apex, status: 'active' });
  const zones = await cloudflare(`/zones?${query}`, { method: 'GET' });
  if (Array.isArray(zones) && zones.length > 0) {
    console.log(`Found active Cloudflare zone ${apex}.`);
    return true;
  }
  console.warn(`Cloudflare zone ${apex} is not visible to this token; custom domain attachment may fail.`);
  return false;
}

async function existingDomains() {
  try {
    const domains = await cloudflare(`${projectPath}/domains`, { method: 'GET' });
    return new Set((domains || []).map((domain) => domain.name));
  } catch (error) {
    if (error.status === 404) return new Set();
    throw error;
  }
}

async function addDomain(domain) {
  try {
    await cloudflare(`${projectPath}/domains`, {
      method: 'POST',
      body: JSON.stringify({ name: domain }),
    });
    console.log(`Added Pages custom domain ${domain}.`);
  } catch (error) {
    const message = String(error.message || '');
    if (message.includes('already') || message.includes('exists')) {
      console.log(`Pages custom domain ${domain} already exists.`);
      return;
    }
    throw error;
  }
}

await ensureZoneVisible();
const project = await ensureProject();
const domains = await existingDomains();
for (const domain of domainList) {
  if (domains.has(domain)) {
    console.log(`Pages custom domain ${domain} already exists.`);
  } else {
    await addDomain(domain);
  }
}
console.log(`Pages production URL: ${project.subdomain || `${projectName}.pages.dev`}`);
