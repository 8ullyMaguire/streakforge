import { s as sanitize_props, a as spread_props, b as slot, i as head } from "../../../chunks/index.js";
import { I as Icon } from "../../../chunks/Icon.js";
function Download($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }],
    ["polyline", { "points": "7 10 12 15 17 10" }],
    ["line", { "x1": "12", "x2": "12", "y1": "15", "y2": "3" }]
  ];
  Icon($$renderer, spread_props([
    { name: "download" },
    $$sanitized_props,
    {
      /**
       * @component @name Download
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMjEgMTV2NGEyIDIgMCAwIDEtMiAySDVhMiAyIDAgMCAxLTItMnYtNCIgLz4KICA8cG9seWxpbmUgcG9pbnRzPSI3IDEwIDEyIDE1IDE3IDEwIiAvPgogIDxsaW5lIHgxPSIxMiIgeDI9IjEyIiB5MT0iMTUiIHkyPSIzIiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/download
       * @see https://lucide.dev/guide/packages/lucide-svelte - Documentation
       *
       * @param {Object} props - Lucide icons props and any valid SVG attribute
       * @returns {FunctionalComponent} Svelte component
       *
       */
      iconNode,
      children: ($$renderer2) => {
        $$renderer2.push(`<!--[-->`);
        slot($$renderer2, $$props, "default", {});
        $$renderer2.push(`<!--]-->`);
      },
      $$slots: { default: true }
    }
  ]));
}
function _page($$renderer) {
  head("2j7mc4", $$renderer, ($$renderer2) => {
    $$renderer2.title(($$renderer3) => {
      $$renderer3.push(`<title>Install — StreakForge</title>`);
    });
  });
  $$renderer.push(`<div class="container" style="max-width:720px;padding-top:28px;"><h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">INSTALL THE APP</h1> <p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">No download needed. StreakForge runs in your browser — add it to your home
		screen and it opens like a real app, with its own icon and no address bar.</p> <div class="card" style="padding:18px 20px;margin-bottom:16px;"><h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🦊 Firefox (Android)</h2> <ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;"><li>Open <a href="/" style="color:var(--red);">streakforge.polarisocial.xyz</a></li> <li>Tap the <strong>three dots</strong> menu (top right)</li> <li>Tap <strong>More</strong></li> <li>Tap <strong>Add app to home screen</strong></li> <li>Confirm — the app icon appears on your home screen</li></ol></div> <div class="card" style="padding:18px 20px;margin-bottom:16px;"><h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🌐 Chrome / Android</h2> <ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;"><li>Open the site</li> <li>Tap the <strong>three dots</strong> menu (top right)</li> <li>Tap <strong>Add to Home screen</strong> (or <strong>Install app</strong>)</li> <li>Tap <strong>Install</strong> / <strong>Add</strong></li></ol></div> <div class="card" style="padding:18px 20px;margin-bottom:16px;"><h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">🍎 iPhone / iPad (Safari)</h2> <ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;"><li>Open the site in Safari</li> <li>Tap the <strong>Share</strong> button (box with arrow, bottom)</li> <li>Tap <strong>Add to Home Screen</strong></li> <li>Tap <strong>Add</strong> (top right)</li></ol></div> <div class="card" style="padding:18px 20px;margin-bottom:16px;"><h2 style="font-size:16px;letter-spacing:0.06em;margin:0 0 8px;">💻 Desktop (any browser)</h2> <ol style="margin:0;padding-left:20px;color:var(--text);font-size:14px;line-height:1.8;"><li>Open the site</li> <li>Look for the <strong>install icon</strong> in the address bar (Chrome/Edge: monitor-with-plus icon)</li> <li>Click it, then <strong>Install</strong></li> <li>Or: Chrome menu → <strong>Cast, save and share</strong> → <strong>Install page as app</strong></li></ol></div> <p style="color:var(--text-dim);font-size:13px;text-align:center;margin-top:20px;">`);
  Download($$renderer, { size: 14, style: "display:inline;vertical-align:-2px;" });
  $$renderer.push(`<!----> The app works offline-ish, remembers you (30 days), and keeps your lock
		streak ticking. Deny harder.</p></div>`);
}
export {
  _page as default
};
