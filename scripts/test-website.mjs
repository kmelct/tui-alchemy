import { readFile, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const cargoToml = await readFile(join(root, 'Cargo.toml'), 'utf8');
const version = cargoToml.match(/^version = "([^"]+)"$/m)?.[1];
if (!version) throw new Error('Cargo.toml package version not found');

const html = await readFile(join(root, 'website', 'index.html'), 'utf8');
const terminalJs = await readFile(join(root, 'website', 'packages', 'web-terminal', 'src', 'index.js'), 'utf8');
const terminalCss = await readFile(join(root, 'website', 'packages', 'web-terminal', 'src', 'terminal.css'), 'utf8');

assert(!html.includes('__bundler'), 'website/index.html must be extracted from the standalone bundler wrapper');
assert(html.includes(`class="ver">v${version}</span>`), 'website footer version must match Cargo.toml');
assert(html.includes('https://i.tui-alchemy.sh'), 'install command must use the installer subdomain');
assert(html.includes('copyTextToClipboard'), 'copy button must use the robust clipboard helper');
assert(html.includes('document.execCommand'), 'copy helper must include a non-secure-context fallback');
assert(html.includes('aria-live="polite"'), 'copy status must be announced to assistive tech');
assert(html.includes('name="description"'), 'website must include a search description');
assert(html.includes('rel="canonical" href="https://tui-alchemy.sh/"'), 'website must declare the canonical custom domain');
assert(html.includes('property="og:image"'), 'website must include a social preview image');
assert(html.includes('name="twitter:card" content="summary_large_image"'), 'website must use a large Twitter/X card');
assert(html.includes('rel="icon"'), 'website must include a favicon');
assert(html.includes('application/ld+json'), 'website must include structured data');
assert(html.includes('SoftwareApplication'), 'structured data must describe the published application');
assert(html.includes('cargo install tui-alchemy'), 'SEO copy must mention the Cargo install path');

assert(html.includes('id="screenshotStage"'), 'gallery stage must be addressable for pointer interaction');
assert(html.includes('data-relic-field'), 'gallery must replace screenshot cards with a pixel-art relic field');
assert(html.includes('data-relic'), 'pixel-art relics must surround the live terminal');
assert(!html.includes('data-terminal-placeholder'), 'landing must not render old screenshot placeholders behind the terminal');
assert(!html.includes('BOOT CARD'), 'landing must not keep old screenshot-card captions');
assert(!html.includes('RUNE ATLAS'), 'landing must not keep old screenshot-card captions');
assert(html.includes('hero-console'), 'hero kicker must be styled as a small terminal shell');
assert(html.includes('--pointer-x'), 'pixel relics must use pointer-driven CSS variables');
assert(html.includes('pointermove'), 'pixel relics must react to pointer movement');
assert(html.includes('requestAnimationFrame'), 'pointer interaction must be frame-coalesced');
assert(html.includes('prefers-reduced-motion'), 'motion must respect reduced-motion users');
assert(html.includes('data-particle-field'), 'background particle canvas must be marked as an animated field');
assert(html.includes('const animateParticles'), 'particle code must choose an explicit animation mode');
assert(html.includes('requestAnimationFrame(frame);'), 'particle field must schedule animation frames');
assert(!html.includes('shooting stars'), 'background must not use the old noisy shooting-star effect');
assert(!html.includes('nebula'), 'background must not use the old blurred AI-style nebula layers');
assert(!html.includes('orb'), 'background must not use the old blurred orb field');

assert(html.includes('id="terminalShell"'), 'landing must include the fantasy web terminal shell');
assert(html.includes('id="alchemyTerminal"'), 'landing must include an xterm.js mount point');
assert(html.includes('id="terminalIntro"'), 'landing must include a terminal loading intro');
assert(html.includes('data-active-terminal'), 'active terminal card must be marked as the live app');
assert(html.includes('terminal.css?v='), 'landing must cache-bust the packaged terminal CSS');
assert(html.includes('terminal.js?v='), 'landing must cache-bust the packaged terminal JS');
assert(html.includes('packages/web-terminal/terminal.css'), 'landing must load the web-terminal package CSS');
assert(html.includes('alchemy_terminal_wasm.wasm?v='), 'landing must cache-bust the packaged WASM binary');
assert(html.includes('packages/web-terminal/terminal.js'), 'landing must load the web-terminal package JS');
assert(html.includes('AlchemyTerminalWasm'), 'landing must expose WASM terminal package config');
assert(html.includes('data-pc-autostart'), 'landing must mark the terminal as an old-PC autostart panel');
assert(html.includes('POWER-ON SELF TEST'), 'terminal intro must use old-PC loading copy');
assert(html.includes('DASH://'), 'landing terminal chrome must use the Dash protocol name');
assert(html.includes('ARCANE MEMORY OK'), 'landing must include the fantasy boot splash text');
assert(!html.includes('WASM ONLINE'), 'landing must not expose the prototype wasm online label');
assert(!html.includes('Terminal reaches active state'), 'landing must not include verification notes in user-facing copy');
assert(!html.includes('combine earth pressure returns STONE'), 'landing must not include verification notes in user-facing copy');
assert(!html.includes('status reports 755'), 'landing must not include verification notes in user-facing copy');
assert(!html.includes('Browser screenshot looked coherent'), 'landing must not include verification notes in user-facing copy');
assert(terminalCss.includes('DASH:// ALCHEMY LINK'), 'terminal chrome must use the Dash protocol name in packaged CSS');
assert(!terminalCss.includes('WEB GRIMOIRE'), 'terminal chrome must not use the prototype Web Grimoire title');
assert(!terminalCss.includes('WASM ONLINE'), 'terminal chrome must not expose the prototype wasm online label');
assert(terminalCss.includes('font-family: "Terminus Nerd Font"'), 'active terminal should prefer Terminus Nerd Font when available');
assert(terminalCss.includes('.terminal-legend'), 'active terminal must include visible user-facing shell instructions');
assert(terminalCss.includes('user-select: none'), 'live demo terminal must disable text selection so drag-and-drop works');
assert(terminalJs.includes('MOUNTING RATATUI WORKSHOP'), 'terminal boot sequence must describe the real ratatui app handoff');
assert(terminalJs.includes('mouse_down'), 'web terminal must forward pointer clicks into the wasm app');
assert(terminalJs.includes('mouse_drag'), 'web terminal must forward pointer drags into the wasm app');
assert(terminalJs.includes('mouse_up'), 'web terminal must forward pointer releases into the wasm app');
assert(terminalJs.includes('key_char'), 'web terminal must forward printable keys into the wasm app');
assert(terminalJs.includes('key_enter'), 'web terminal must forward Enter into the wasm app');
assert(terminalJs.includes('\\x1b[?1049h'), 'web terminal must switch xterm into the alternate screen for the live app');
assert(terminalJs.includes('setPointerCapture'), 'pointer dragging must keep capture while dragging across the live app');
assert(terminalJs.includes('scrollback: 0'), 'live demo terminal must disable scrollback so the ratatui app stays in-place');
assert(!terminalJs.includes('Local shell command: tui-alchemy'), 'web demo must not render the old fake shell startup text');
assert(!terminalJs.includes('submit_input'), 'web demo must not use the old line-command shell api');
assert(!terminalJs.includes('response_ptr'), 'web demo must not use the old line-command shell api');
assert(!terminalJs.includes('redrawInput'), 'web demo must not keep the old shell prompt editor');
assert(!terminalJs.includes('cursorBlink: true'), 'web terminal must not use a blinking cursor effect');
assert(!terminalJs.includes('bindPointerParallax(shell)'), 'active terminal must not use the shell shake effect');
assert(!terminalJs.includes('card-breathe'), 'active terminal hover stack must not use the old screenshot blinking/breathing animation');



const assetRefs = [...html.matchAll(/(?:src|href)="(assets\/[^"]+)"/g)].map((match) => match[1]);
assert(assetRefs.length >= 8, 'website should reference extracted local assets');
for (const ref of assetRefs) {
  await stat(join(root, 'website', ref));
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

const unixInstaller = await readFile(join(root, 'scripts', 'install-tui-alchemy.sh'), 'utf8');
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

const windowsInstaller = await readFile(join(root, 'scripts', 'install-tui-alchemy.ps1'), 'utf8');
assert(windowsInstaller.includes('Install-FromBinary'), 'Windows installer must try a prebuilt binary first');
assert(windowsInstaller.includes('BinaryBaseUrl'), 'Windows installer must have a binary asset base URL');
assert(windowsInstaller.includes('cargo install $AppName --version $Version --locked --force'), 'Windows installer must fall back to the published crates.io package');
assert(windowsInstaller.includes('Install-MissingDependency'), 'Windows installer must prompt-install missing runtime dependencies');
assert(windowsInstaller.includes('Prompt-YesNo'), 'Windows installer must ask before installing missing dependencies');
assert(windowsInstaller.includes('winget'), 'Windows installer must detect the native package manager when dependencies are missing');
assert(!windowsInstaller.includes('rustup-init'), 'Windows installer must not bootstrap Rust');
assert(!windowsInstaller.includes('Install-Rust'), 'Windows installer must not install Rust');
assert(!windowsInstaller.toLowerCase().includes('python'), 'Windows installer must not require Python');

const installDocs = await readFile(join(root, 'docs', 'install.md'), 'utf8');
assert(installDocs.includes('cargo install tui-alchemy --locked'), 'install docs must include the Cargo Book recommended locked install');
assert(installDocs.includes('cargo publish --dry-run'), 'install docs must mention dry-run before publishing');
assert(installDocs.includes('cargo package --list'), 'install docs must mention package contents review');
assert(installDocs.includes('CARGO_HOME'), 'install docs must explain Cargo install root behavior');

const buildScript = await readFile(join(root, 'scripts', 'build-website.sh'), 'utf8');
assert(buildScript.includes('npm --prefix website run build'), 'build script must build website packages');
assert(buildScript.includes('npm --prefix website ci') || buildScript.includes('npm --prefix website install'), 'build script must install website package dependencies');

const websitePackage = await readFile(join(root, 'website', 'package.json'), 'utf8');
assert(websitePackage.includes('packages/alchemy-wasm/Cargo.toml'), 'website package build must compile the WASM package');
assert(websitePackage.includes('@xterm/xterm'), 'website package must depend on xterm.js');

const deployScript = await readFile(join(root, 'scripts', 'deploy-website.sh'), 'utf8');
assert(deployScript.includes('scripts/provision-cloudflare-pages.mjs'), 'deploy script must provision Pages before deploying');
assert(deployScript.includes('scripts/upload-r2-assets.mjs'), 'deploy script must upload R2 assets');
assert(deployScript.includes('pages deploy website/dist'), 'deploy script must deploy the built Pages output');
assert(deployScript.includes('e9c375806f33a6c2a42c7d5ca9729105'), 'deploy script must default to the Personal account');
assert(deployScript.includes('docker info'), 'deploy script must skip Docker packaging when the Docker daemon is unavailable');
assert(deployScript.includes('Docker is installed but the daemon is unavailable'), 'deploy script must explain when Linux Docker packaging is skipped');

const pagesProvisioner = await readFile(join(root, 'scripts', 'provision-cloudflare-pages.mjs'), 'utf8');
assert(pagesProvisioner.includes("type: 'github'"), 'Pages provisioner must try the native GitHub source first');
assert(pagesProvisioner.includes('Direct Upload project'), 'Pages provisioner must fall back when Cloudflare GitHub integration fails');
assert(pagesProvisioner.includes("build_command: 'sh scripts/build-website.sh'"), 'Pages provisioner must set the website build command');
assert(pagesProvisioner.includes("destination_dir: 'website/dist'"), 'Pages provisioner must deploy the website/dist output');

const wranglerToml = await readFile(join(root, 'wrangler.toml'), 'utf8');
assert(wranglerToml.includes('pages_build_output_dir = "website/dist"'), 'Wrangler config must point at website/dist');

const deployWorkflow = await readFile(join(root, '.github', 'workflows', 'deploy-website.yml'), 'utf8');
assert(deployWorkflow.includes('npx wrangler@latest pages deploy website/dist'), 'CI must deploy website/dist to Cloudflare Pages');
assert(deployWorkflow.includes('node scripts/upload-r2-assets.mjs'), 'CI must upload installer assets to R2');

const dnsConfigurator = await readFile(join(root, 'scripts', 'configure-cloudflare-dns.mjs'), 'utf8');
assert(dnsConfigurator.includes("type: 'CNAME'"), 'DNS configurator must create CNAME records');
assert(dnsConfigurator.includes('www.${zoneName}'), 'DNS configurator must manage the www host');

const pagesWorker = await readFile(join(root, 'website', 'dist', '_worker.js'), 'utf8');
assert(pagesWorker.includes('env.ASSETS.fetch(request)'), 'Pages worker must pass website requests to static assets');
assert(pagesWorker.includes('i.tui-alchemy.sh'), 'Pages worker must route installer subdomain requests');
assert(pagesWorker.includes('www.tui-alchemy.sh'), 'Pages worker must recognize the www host');
assert(pagesWorker.includes('tui-alchemy.pages.dev'), 'Pages worker must recognize the default Pages host');
assert(pagesWorker.includes('.tui-alchemy.pages.dev'), 'Pages worker must recognize preview Pages hosts');
assert(pagesWorker.includes('Response.redirect(url.toString(), 301)'), 'Pages worker must redirect non-canonical public hosts permanently');

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
