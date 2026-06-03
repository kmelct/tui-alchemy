import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';

let decoder = new TextDecoder();
const TICK_MS = 120;

function readWasmString(instance, ptrName, lenName) {
  const { memory } = instance.exports;
  const ptr = instance.exports[ptrName]();
  const len = instance.exports[lenName]();
  return decoder.decode(new Uint8Array(memory.buffer, ptr, len));
}

function readScreen(instance) {
  return readWasmString(instance, 'screen_ptr', 'screen_len');
}

async function instantiateWasm(url) {
  const response = await fetch(url, { cache: 'no-store' });
  if (!response.ok) throw new Error(`WASM fetch failed: ${response.status}`);
  if (WebAssembly.instantiateStreaming && response.headers.get('content-type') === 'application/wasm') {
    return (await WebAssembly.instantiateStreaming(response, {})).instance;
  }
  return (await WebAssembly.instantiate(await response.arrayBuffer(), {})).instance;
}

function setStatus(shell, text) {
  const status = shell.querySelector('[data-terminal-status]');
  if (status) status.textContent = text;
}

async function playBootSequence(intro, shell, instancePromise) {
  if (!intro) {
    await instancePromise;
    return;
  }
  const screen = intro.querySelector('pre');
  const hint = intro.querySelector('.intro-hint');
  if (screen) {
    screen.textContent = [
      'POWER-ON SELF TEST',
      'DASH:// BIOS 0.2',
      'ARCANE MEMORY OK',
      'MOUNTING RATATUI WORKSHOP',
      'LOADING ALCHEMY TUI',
    ].join('\n');
  }
  if (hint) hint.textContent = 'STARTING LIVE DEMO';
  setStatus(shell, 'POST RUNNING');
  await instancePromise;
  if (hint) hint.textContent = 'HANDING OFF TO RATATUI';
  setStatus(shell, 'WORKSHOP READY');
  await new Promise((resolve) => window.setTimeout(resolve, 220));
}

function renderFrame(term, instance) {
  term.write(readScreen(instance));
}

function syncSize(fit, term, instance) {
  fit.fit();
  instance.exports.resize(term.cols, term.rows);
  renderFrame(term, instance);
}

function pointerToCell(event, term, surface) {
  const rect = surface.getBoundingClientRect();
  const x = Math.max(0, Math.min(term.cols - 1, Math.floor(((event.clientX - rect.left) / rect.width) * term.cols)));
  const y = Math.max(0, Math.min(term.rows - 1, Math.floor(((event.clientY - rect.top) / rect.height) * term.rows)));
  return { x, y };
}

function bindPointer(instance, term, mount) {
  const surface = mount.querySelector('.xterm-screen') ?? mount;
  surface.style.userSelect = 'none';
  surface.style.touchAction = 'none';

  let dragging = false;
  let lastCell = null;

  const updateDrag = (event, action) => {
    const cell = pointerToCell(event, term, surface);
    if (lastCell && lastCell.x === cell.x && lastCell.y === cell.y && action === 'drag') {
      return;
    }
    lastCell = cell;
    if (action === 'down') {
      instance.exports.mouse_down(cell.x, cell.y);
    } else if (action === 'drag') {
      instance.exports.mouse_drag(cell.x, cell.y);
    } else {
      instance.exports.mouse_up(cell.x, cell.y);
    }
    renderFrame(term, instance);
  };

  surface.addEventListener('pointerdown', (event) => {
    if (event.button !== 0) return;
    dragging = true;
    lastCell = null;
    surface.setPointerCapture(event.pointerId);
    term.focus();
    updateDrag(event, 'down');
  });

  surface.addEventListener('pointermove', (event) => {
    if (!dragging) return;
    updateDrag(event, 'drag');
  });

  const finish = (event) => {
    if (!dragging) return;
    dragging = false;
    updateDrag(event, 'up');
    if (surface.hasPointerCapture(event.pointerId)) {
      surface.releasePointerCapture(event.pointerId);
    }
  };

  surface.addEventListener('pointerup', finish);
  surface.addEventListener('pointercancel', finish);
}

function dispatchNamedKey(instance, name) {
  switch (name) {
    case 'up':
      instance.exports.key_up();
      return true;
    case 'down':
      instance.exports.key_down();
      return true;
    case 'left':
      instance.exports.key_left();
      return true;
    case 'right':
      instance.exports.key_right();
      return true;
    case 'pageUp':
      instance.exports.key_page_up();
      return true;
    case 'pageDown':
      instance.exports.key_page_down();
      return true;
    case 'home':
      instance.exports.key_home();
      return true;
    case 'end':
      instance.exports.key_end();
      return true;
    case 'enter':
      instance.exports.key_enter();
      return true;
    case 'escape':
      instance.exports.key_escape();
      return true;
    case 'backspace':
      instance.exports.key_backspace();
      return true;
    default:
      return false;
  }
}

function bindKeyboard(instance, term) {
  term.onData((data) => {
    let changed = false;
    const trigger = (name) => {
      changed = dispatchNamedKey(instance, name) || changed;
    };

    switch (data) {
      case '\r':
        trigger('enter');
        break;
      case '\x1b':
        trigger('escape');
        break;
      case '\x7f':
      case '\b':
        trigger('backspace');
        break;
      case '\x1b[A':
      case '\x1bOA':
        trigger('up');
        break;
      case '\x1b[B':
      case '\x1bOB':
        trigger('down');
        break;
      case '\x1b[C':
      case '\x1bOC':
        trigger('right');
        break;
      case '\x1b[D':
      case '\x1bOD':
        trigger('left');
        break;
      case '\x1b[5~':
        trigger('pageUp');
        break;
      case '\x1b[6~':
        trigger('pageDown');
        break;
      case '\x1b[H':
      case '\x1bOH':
        trigger('home');
        break;
      case '\x1b[F':
      case '\x1bOF':
        trigger('end');
        break;
      default:
        for (const ch of data) {
          if (ch >= ' ' && ch <= '~') {
            instance.exports.key_char(ch.codePointAt(0));
            changed = true;
          }
        }
        break;
    }

    if (changed) {
      renderFrame(term, instance);
    }
  });
}

async function bootTerminal() {
  const config = window.AlchemyTerminalWasm;
  const mount = document.getElementById('alchemyTerminal');
  const shell = document.getElementById('terminalShell');
  const intro = document.getElementById('terminalIntro');
  if (!config || !mount || !shell) return;

  shell.dataset.terminalState = 'loading';
  setStatus(shell, 'POST RUNNING');

  const term = new Terminal({
    allowTransparency: true,
    convertEol: false,
    cursorBlink: false,
    cursorStyle: 'block',
    drawBoldTextInBrightColors: false,
    fontFamily: '"Terminus Nerd Font", "Terminus", VT323, ui-monospace, SFMono-Regular, Menlo, monospace',
    fontSize: 14,
    letterSpacing: 0,
    lineHeight: 1,
    scrollback: 0,
    theme: {
      background: '#08090f',
      foreground: '#d2c9a5',
      cursor: '#d6c97a',
      selectionBackground: '#5c4226aa',
      black: '#08090f',
      red: '#c77b58',
      green: '#8caba1',
      yellow: '#d6c97a',
      blue: '#7f9fc7',
      magenta: '#9a7cc4',
      cyan: '#56a29c',
      white: '#d2c9a5',
      brightBlack: '#383e58',
      brightRed: '#c77b58',
      brightGreen: '#8caba1',
      brightYellow: '#f1df8c',
      brightBlue: '#b6d4ff',
      brightMagenta: '#d7c5ff',
      brightCyan: '#8fd6cd',
      brightWhite: '#fff8df',
    },
  });
  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open(mount);
  fit.fit();

  const instancePromise = instantiateWasm(config.wasmUrl);
  try {
    await playBootSequence(intro, shell, instancePromise);
  } catch (error) {
    shell.dataset.terminalState = 'error';
    setStatus(shell, 'BOOT FAULT');
    term.write(`\r\n\x1b[31m${error.message}\x1b[0m\r\n`);
    return;
  }

  const instance = await instancePromise;
  term.write('\x1b[?1049h\x1b[?25l');
  syncSize(fit, term, instance);
  bindKeyboard(instance, term);
  bindPointer(instance, term, mount);

  const resize = () => syncSize(fit, term, instance);
  window.addEventListener('resize', resize, { passive: true });

  window.setInterval(() => {
    instance.exports.tick();
    renderFrame(term, instance);
  }, TICK_MS);

  shell.dataset.terminalState = 'active';
  setStatus(shell, 'LIVE DEMO READY');
  if (intro) intro.dataset.loaded = 'true';
  term.focus();
}

bootTerminal();
