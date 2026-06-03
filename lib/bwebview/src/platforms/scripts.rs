/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) const IPC_SCRIPT: &str = r#"try { window.ipc = new EventTarget(); } catch (e) {
  window.ipc = { _h: {},
    addEventListener(t, l) { (this._h[t] = this._h[t] || []).push(l); },
    removeEventListener(t, l) { this._h[t] = (this._h[t] || []).filter(x => x !== l); },
    dispatchEvent(e) { (this._h[e.type] || []).forEach(l => l(e)); return true; }
  };
}
if (window.webkit) {
  window.ipc.postMessage = m =>
    window.webkit.messageHandlers.ipc.postMessage(typeof m !== 'string' ? JSON.stringify(m) : m);
} else {
  window.ipc.postMessage = m =>
    window.chrome.webview.postMessage('i' + (typeof m !== 'string' ? JSON.stringify(m) : m));
}
"#;

#[cfg(feature = "log")]
pub(crate) const CONSOLE_SCRIPT: &str = r#"for (const level of ['error', 'warn', 'info', 'debug', 'trace', 'log']) {
  window.console[level] = (...args) => {
    const msg = args.map(arg => typeof arg !== 'string' ? JSON.stringify(arg) : arg).join(' ');
    if (window.webkit) {
      window.webkit.messageHandlers.console.postMessage(level.charAt(0) + msg);
    } else {
      window.chrome.webview.postMessage('c' + level.charAt(0) + msg);
    }
  };
}
"#;
