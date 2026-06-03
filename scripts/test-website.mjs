import { readFile, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const cargoToml = await readFile(join(root, 'Cargo.toml'), 'utf8');
const version = cargoToml.match(/^version = "([^"]+)"$/m)?.[1];
if (!version) throw new Error('Cargo.toml package version not found');

const read = (rel) => readFile(join(root, rel), 'utf8');

// The landing page is a single static index.html + a linked stylesheet, plus the
// real ratatui game compiled to wasm and a small xterm.js bridge. No framework.
const html = await read('website/index.html');
const css = await read('website/assets/main.css');
const terminalJs = await read('website/packages/web-terminal/src/index.js');
const terminalCss = await read('website/packages/web-terminal/src/terminal.css');
const buildScript = await read('scripts/build-website.sh');

// ---- SEO head ----
assert(html.includes('<title>Alchemy TUI'), 'page must set a document title');
assert(html.includes('name="description"'), 'page must include a search description');
assert(html.includes('rel="canonical" href="https://tui-alchemy.sh/"'), 'page must declare the canonical domain');
assert(html.includes('property="og:image"'), 'page must include a social preview image');
assert(html.includes('name="twitter:card" content="summary_large_image"'), 'page must use a large Twitter/X card');
assert(html.includes('rel="icon"'), 'page must include a favicon');
assert(html.includes('application/ld+json'), 'page must include structured data');
assert(html.includes('SoftwareApplication'), 'structured data must describe the published application');
assert(html.includes('/assets/main.css'), 'page must link the design stylesheet');
assert(html.includes('/assets/terminal.css'), 'page must link the terminal stylesheet');
assert(html.includes('/assets/terminal.js'), 'page must load the xterm bridge');

// ---- real, specific page content ----
assert(html.includes('https://i.tui-alchemy.sh'), 'install command must use the installer subdomain');
assert(html.includes('cargo install tui-alchemy'), 'page must mention the Cargo install path');
assert(html.includes('755'), 'tagline must state the real 755-element goal');
assert(html.includes('hjkl'), 'controls copy must state the real movement keys');
assert(html.includes('to quit'), 'copy must state the real quit key');
assert(html.includes('live vm'), 'the demo caption must be the minimal live-vm label');
assert(html.includes('AlchemyFX'), 'the page must define the retro FX (sound + keyboard flash)');
assert(html.includes('rig-kbd'), 'the page must include the C64 keyboard flash overlay');
assert(html.includes(`class="ver">v${version}</span>`), 'footer version must match Cargo.toml');
assert(html.includes('copyTextToClipboard'), 'copy button must use the robust clipboard helper');
assert(html.includes('document.execCommand'), 'copy helper must include a non-secure-context fallback');
assert(html.includes('aria-live="polite"'), 'copy status must be announced to assistive tech');
assert(html.includes('id="terminalShell"'), 'page must include the live terminal shell');
assert(html.includes('id="alchemyTerminal"'), 'page must include the xterm mount point');
assert(html.includes('id="terminalIntro"'), 'page must include the terminal loading intro');
assert(html.includes('data-active-terminal'), 'live terminal must be marked as the active app');
assert(html.includes('data-pc-autostart'), 'page must autostart the live demo');
assert(html.includes('AlchemyTerminalWasm'), 'page must expose the wasm demo config');
assert(html.includes('/assets/sprites/'), 'recipe formula must use real element sprites');
assert(html.includes('/assets/gen/retro-computer.png'), 'live demo must be housed in the retro computer artwork');
assert(html.includes('/assets/gen/wax-seal.png'), "parchment card must carry the maker's wax seal");

// ---- the removed AI-slop must stay gone (regression guard) ----
assert(!html.toLowerCase().includes('dash://'), 'the fake dash:// protocol must not return');
assert(!html.includes('hero-console'), 'the fake terminal kicker block must not return');
assert(!html.includes('POWER-ON SELF TEST'), 'the fake BIOS boot text must not return');
assert(!html.includes('ARCANE MEMORY OK'), 'the fake memory-check boot text must not return');
assert(!html.includes('data-relic'), 'the decorative relic parallax field must not return');
assert(!html.includes('data-particle-field'), 'the background particle canvas must not return');

// ---- the scene + restraint (CSS) ----
assert(css.includes('workshop-backdrop'), 'the scene must use the generated workshop backdrop');
assert(css.includes('parchment-sheet'), 'the copy must sit on a parchment surface');
assert(css.includes('recipeCycle'), 'the recipe scroll must cross-fade real recipes');
assert(css.includes('kbdpress'), 'the C64 keyboard must flash when keys are pressed');
assert(css.includes('prefers-reduced-motion'), 'motion must respect reduced-motion users');
assert(!css.includes('data-relic') && !css.includes('data-particle-field'), 'the old effect chrome must not return');

// ---- honest boot + live demo input wiring (xterm bridge) ----
assert(terminalJs.includes('loading alchemy.wasm'), 'terminal boot must show the honest wasm load line');
assert(!terminalJs.includes('MOUNTING RATATUI WORKSHOP') && !terminalJs.includes('POWER-ON SELF TEST'), 'terminal boot must not keep the fake BIOS sequence');
assert(terminalJs.includes('waitForEl'), 'the bridge must wait for the terminal mount point');
assert(terminalJs.includes('AlchemyFX'), 'the bridge must trigger retro FX (sound + keyboard flash) on key/boot');
assert(terminalJs.includes('mouse_down') && terminalJs.includes('mouse_drag') && terminalJs.includes('mouse_up'), 'web terminal must forward pointer events into the wasm app');
assert(terminalJs.includes('key_char') && terminalJs.includes('key_enter'), 'web terminal must forward keys into the wasm app');
assert(terminalJs.includes('\\x1b[?1049h'), 'web terminal must switch xterm into the alternate screen');
assert(terminalJs.includes('scrollback: 0'), 'live demo terminal must disable scrollback');
assert(!terminalJs.includes('cursorBlink: true'), 'web terminal must not use a blinking cursor effect');
assert(!terminalCss.includes('DASH://'), 'the live screen must not invent a dash:// link title');
assert(terminalCss.includes('.terminal-window'), 'the live screen must render the terminal window surface');
assert(terminalCss.includes('user-select: none'), 'live demo terminal must disable text selection so drag-and-drop works');

// ---- build pipeline (static: copy + wasm + esbuild, no framework) ----
assert(!buildScript.includes('dx build') && !buildScript.includes('dioxus'), 'build script must be a plain static build, not a framework build');
assert(buildScript.includes('cp website/index.html'), 'build script must publish the static page');
assert(buildScript.includes('wasm32-unknown-unknown'), 'build script must compile the game to wasm');
assert(buildScript.includes('alchemy_terminal_wasm.wasm'), 'build script must publish the game wasm');
assert(buildScript.includes('esbuild'), 'build script must bundle the xterm bridge');

// ---- referenced assets exist ----
const assetRefs = [...html.matchAll(/(?:src|href)="(\/assets\/[^"?]+)"/g)].map((m) => m[1]);
assert(assetRefs.length >= 5, 'page should reference local assets');
for (const ref of assetRefs) {
  await stat(join(root, 'website', ref.replace(/^\//, '')));
}
for (const el of ['water', 'fire', 'steam', 'earth', 'lava', 'air', 'rain']) {
  await stat(join(root, 'website', 'assets', 'sprites', `${el}.png`));
}
// the Open Graph image is referenced as an absolute URL, so stat it explicitly
await stat(join(root, 'website', 'assets', 'social-preview.png'));

// ---- the built static output (run after build-website.sh) ----
let built = null;
try {
  built = await read('website/dist/index.html');
} catch (error) {
  if (error?.code !== 'ENOENT') throw error;
  // website/dist not built yet — skip the built-output checks.
}
if (built !== null) {
  assert(built.includes('<title>Alchemy TUI'), 'built page must keep the static title');
  assert(built.includes('SoftwareApplication'), 'built page must keep structured data');
  assert(built.includes('AlchemyTerminalWasm'), 'built page must wire the live demo');
  // The live demo is the hero — its real artifacts must be present in the deployable output,
  // not just referenced. (A missing one here means CI would ship a dead demo.)
  for (const artifact of ['assets/terminal.js', 'assets/terminal.css', 'assets/main.css', '_worker.js']) {
    const info = await stat(join(root, 'website', 'dist', artifact));
    assert(info.size > 0, `built dist/${artifact} must be non-empty`);
  }
  const wasm = await stat(join(root, 'website', 'dist', 'assets', 'alchemy_terminal_wasm.wasm'));
  assert(wasm.size > 100_000, `built game wasm looks too small (${wasm.size} bytes) — demo would be broken`);
}

for (const file of [
  'scripts/build-website.sh',
  'scripts/install-tui-alchemy.sh',
  'scripts/install-tui-alchemy.ps1',
  'scripts/deploy-website.sh',
  'scripts/provision-cloudflare-pages.mjs',
  'scripts/upload-r2-assets.mjs',
  'scripts/configure-cloudflare-dns.mjs',
  'scripts/package-current-binary.mjs',
  'scripts/package-linux-binary-in-docker.sh',
  'scripts/test-installer-docker.sh',
  'website/index.html',
  'website/assets/main.css',
  'website/package.json',
  'website/packages/alchemy-wasm/Cargo.toml',
  'website/packages/alchemy-wasm/src/lib.rs',
  'website/packages/web-terminal/src/index.js',
  'website/packages/web-terminal/src/terminal.css',
  'docs/install.md',
  'wrangler.toml',
  '.github/workflows/deploy-website.yml',
]) {
  await stat(join(root, file));
}

const unixInstaller = await read('scripts/install-tui-alchemy.sh');
assert(unixInstaller.includes('install_from_binary'), 'Unix installer must try a prebuilt binary first');
assert(unixInstaller.includes('BINARY_BASE_URL'), 'Unix installer must have a binary asset base URL');
assert(unixInstaller.includes('ensure_download_tool'), 'Unix installer must detect missing download tools');
assert(unixInstaller.includes('ensure_archive_tool'), 'Unix installer must detect missing archive tools');
assert(unixInstaller.includes('install_missing_dependency'), 'Unix installer must prompt-install missing runtime dependencies');
assert(unixInstaller.includes('apt-get') && unixInstaller.includes('dnf') && unixInstaller.includes('brew') && unixInstaller.includes('pacman'), 'Unix installer must know common package managers');
assert(unixInstaller.includes('TUI_ALCHEMY_YES'), 'Unix installer must support non-interactive approval');
assert(unixInstaller.includes('cargo install "$APP_NAME" --version "$APP_VERSION" --locked --force'), 'Unix installer must fall back to the published crates.io package');
assert(!unixInstaller.includes('ensure_rust'), 'Unix installer must not require Rust for the primary install path');
assert(!unixInstaller.includes('rustup'), 'Unix installer must not bootstrap Rust');
assert(!unixInstaller.includes('python'), 'Unix installer must not require Python');

const windowsInstaller = await read('scripts/install-tui-alchemy.ps1');
assert(windowsInstaller.includes('Install-FromBinary'), 'Windows installer must try a prebuilt binary first');
assert(windowsInstaller.includes('BinaryBaseUrl'), 'Windows installer must have a binary asset base URL');
assert(windowsInstaller.includes('cargo install $AppName --version $Version --locked --force'), 'Windows installer must fall back to the published crates.io package');
assert(windowsInstaller.includes('Install-MissingDependency'), 'Windows installer must prompt-install missing runtime dependencies');
assert(windowsInstaller.includes('Prompt-YesNo'), 'Windows installer must ask before installing missing dependencies');
assert(windowsInstaller.includes('winget'), 'Windows installer must detect the native package manager when dependencies are missing');
assert(!windowsInstaller.includes('rustup-init'), 'Windows installer must not bootstrap Rust');
assert(!windowsInstaller.includes('Install-Rust'), 'Windows installer must not install Rust');
assert(!windowsInstaller.toLowerCase().includes('python'), 'Windows installer must not require Python');

const installDocs = await read('docs/install.md');
assert(installDocs.includes('cargo install tui-alchemy --locked'), 'install docs must include the Cargo Book recommended locked install');
assert(installDocs.includes('cargo publish --dry-run'), 'install docs must mention dry-run before publishing');
assert(installDocs.includes('cargo package --list'), 'install docs must mention package contents review');
assert(installDocs.includes('CARGO_HOME'), 'install docs must explain Cargo install root behavior');

const websitePackage = await read('website/package.json');
assert(websitePackage.includes('@xterm/xterm'), 'website package must depend on xterm.js');
assert(websitePackage.includes('esbuild'), 'website package must provide esbuild for the terminal bridge');

const deployScript = await read('scripts/deploy-website.sh');
assert(deployScript.includes('scripts/provision-cloudflare-pages.mjs'), 'deploy script must provision Pages before deploying');
assert(deployScript.includes('scripts/upload-r2-assets.mjs'), 'deploy script must upload R2 assets');
assert(deployScript.includes('pages deploy website/dist'), 'deploy script must deploy the built Pages output');
assert(deployScript.includes('e9c375806f33a6c2a42c7d5ca9729105'), 'deploy script must default to the Personal account');
assert(deployScript.includes('docker info'), 'deploy script must skip Docker packaging when the Docker daemon is unavailable');

const pagesProvisioner = await read('scripts/provision-cloudflare-pages.mjs');
assert(pagesProvisioner.includes("type: 'github'"), 'Pages provisioner must try the native GitHub source first');
assert(pagesProvisioner.includes('Direct Upload project'), 'Pages provisioner must fall back when Cloudflare GitHub integration fails');
assert(pagesProvisioner.includes("build_command: 'sh scripts/build-website.sh'"), 'Pages provisioner must set the website build command');
assert(pagesProvisioner.includes("destination_dir: 'website/dist'"), 'Pages provisioner must deploy the website/dist output');

const wranglerToml = await read('wrangler.toml');
assert(wranglerToml.includes('pages_build_output_dir = "website/dist"'), 'Wrangler config must point at website/dist');

const deployWorkflow = await read('.github/workflows/deploy-website.yml');
assert(deployWorkflow.includes('npx wrangler@latest pages deploy website/dist'), 'CI must deploy website/dist to Cloudflare Pages');
assert(deployWorkflow.includes('node scripts/upload-r2-assets.mjs'), 'CI must upload installer assets to R2');
assert(!deployWorkflow.includes('dioxus'), 'CI must not depend on a framework toolchain for a static build');

const dnsConfigurator = await read('scripts/configure-cloudflare-dns.mjs');
assert(dnsConfigurator.includes("type: 'CNAME'"), 'DNS configurator must create CNAME records');
assert(dnsConfigurator.includes('www.${zoneName}'), 'DNS configurator must manage the www host');

const pagesWorker = await read('website/_worker.js');
assert(pagesWorker.includes('env.ASSETS.fetch(request)'), 'Pages worker must pass website requests to static assets');
assert(pagesWorker.includes('i.tui-alchemy.sh'), 'Pages worker must route installer subdomain requests');
assert(pagesWorker.includes('www.tui-alchemy.sh'), 'Pages worker must recognize the www host');
assert(pagesWorker.includes('tui-alchemy.pages.dev'), 'Pages worker must recognize the default Pages host');
assert(pagesWorker.includes('Response.redirect(url.toString(), 301)'), 'Pages worker must redirect non-canonical public hosts permanently');

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
