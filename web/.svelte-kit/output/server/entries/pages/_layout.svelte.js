import { s as sanitize_props, a as spread_props, b as slot, c as ensure_array_like, d as attr, f as attr_class, e as escape_html } from "../../chunks/index.js";
import { p as page } from "../../chunks/index2.js";
import { I as Icon } from "../../chunks/Icon.js";
import "clsx";
let toasts = [];
function getToasts() {
  return toasts;
}
function Ban($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["circle", { "cx": "12", "cy": "12", "r": "10" }],
    ["path", { "d": "m4.9 4.9 14.2 14.2" }]
  ];
  Icon($$renderer, spread_props([
    { name: "ban" },
    $$sanitized_props,
    {
      /**
       * @component @name Ban
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIgLz4KICA8cGF0aCBkPSJtNC45IDQuOSAxNC4yIDE0LjIiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/ban
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
function Flame($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      {
        "d": "M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "flame" },
    $$sanitized_props,
    {
      /**
       * @component @name Flame
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNOC41IDE0LjVBMi41IDIuNSAwIDAgMCAxMSAxMmMwLTEuMzgtLjUtMi0xLTMtMS4wNzItMi4xNDMtLjIyNC00LjA1NCAyLTYgLjUgMi41IDIgNC45IDQgNi41IDIgMS42IDMgMy41IDMgNS41YTcgNyAwIDEgMS0xNCAwYzAtMS4xNTMuNDMzLTIuMjk0IDEtM2EyLjUgMi41IDAgMCAwIDIuNSAyLjV6IiAvPgo8L3N2Zz4K) - https://lucide.dev/icons/flame
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
function House($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    [
      "path",
      { "d": "M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8" }
    ],
    [
      "path",
      {
        "d": "M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "house" },
    $$sanitized_props,
    {
      /**
       * @component @name House
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTUgMjF2LThhMSAxIDAgMCAwLTEtMWgtNGExIDEgMCAwIDAtMSAxdjgiIC8+CiAgPHBhdGggZD0iTTMgMTBhMiAyIDAgMCAxIC43MDktMS41MjhsNy01Ljk5OWEyIDIgMCAwIDEgMi41ODIgMGw3IDUuOTk5QTIgMiAwIDAgMSAyMSAxMHY5YTIgMiAwIDAgMS0yIDJINWEyIDIgMCAwIDEtMi0yeiIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/house
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
function Pen_line($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M12 20h9" }],
    [
      "path",
      {
        "d": "M16.376 3.622a1 1 0 0 1 3.002 3.002L7.368 18.635a2 2 0 0 1-.855.506l-2.872.838a.5.5 0 0 1-.62-.62l.838-2.872a2 2 0 0 1 .506-.854z"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "pen-line" },
    $$sanitized_props,
    {
      /**
       * @component @name PenLine
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTIgMjBoOSIgLz4KICA8cGF0aCBkPSJNMTYuMzc2IDMuNjIyYTEgMSAwIDAgMSAzLjAwMiAzLjAwMkw3LjM2OCAxOC42MzVhMiAyIDAgMCAxLS44NTUuNTA2bC0yLjg3Mi44MzhhLjUuNSAwIDAgMS0uNjItLjYybC44MzgtMi44NzJhMiAyIDAgMCAxIC41MDYtLjg1NHoiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/pen-line
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
function Scroll_text($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M15 12h-5" }],
    ["path", { "d": "M15 8h-5" }],
    ["path", { "d": "M19 17V5a2 2 0 0 0-2-2H4" }],
    [
      "path",
      {
        "d": "M8 21h12a2 2 0 0 0 2-2v-1a1 1 0 0 0-1-1H11a1 1 0 0 0-1 1v1a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v2a1 1 0 0 0 1 1h3"
      }
    ]
  ];
  Icon($$renderer, spread_props([
    { name: "scroll-text" },
    $$sanitized_props,
    {
      /**
       * @component @name ScrollText
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTUgMTJoLTUiIC8+CiAgPHBhdGggZD0iTTE1IDhoLTUiIC8+CiAgPHBhdGggZD0iTTE5IDE3VjVhMiAyIDAgMCAwLTItMkg0IiAvPgogIDxwYXRoIGQ9Ik04IDIxaDEyYTIgMiAwIDAgMCAyLTJ2LTFhMSAxIDAgMCAwLTEtMUgxMWExIDEgMCAwIDAtMSAxdjFhMiAyIDAgMSAxLTQgMFY1YTIgMiAwIDEgMC00IDB2MmExIDEgMCAwIDAgMSAxaDMiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/scroll-text
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
function Trophy($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M6 9H4.5a2.5 2.5 0 0 1 0-5H6" }],
    ["path", { "d": "M18 9h1.5a2.5 2.5 0 0 0 0-5H18" }],
    ["path", { "d": "M4 22h16" }],
    [
      "path",
      {
        "d": "M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22"
      }
    ],
    [
      "path",
      {
        "d": "M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22"
      }
    ],
    ["path", { "d": "M18 2H6v7a6 6 0 0 0 12 0V2Z" }]
  ];
  Icon($$renderer, spread_props([
    { name: "trophy" },
    $$sanitized_props,
    {
      /**
       * @component @name Trophy
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNNiA5SDQuNWEyLjUgMi41IDAgMCAxIDAtNUg2IiAvPgogIDxwYXRoIGQ9Ik0xOCA5aDEuNWEyLjUgMi41IDAgMCAwIDAtNUgxOCIgLz4KICA8cGF0aCBkPSJNNCAyMmgxNiIgLz4KICA8cGF0aCBkPSJNMTAgMTQuNjZWMTdjMCAuNTUtLjQ3Ljk4LS45NyAxLjIxQzcuODUgMTguNzUgNyAyMC4yNCA3IDIyIiAvPgogIDxwYXRoIGQ9Ik0xNCAxNC42NlYxN2MwIC41NS40Ny45OC45NyAxLjIxQzE2LjE1IDE4Ljc1IDE3IDIwLjI0IDE3IDIyIiAvPgogIDxwYXRoIGQ9Ik0xOCAySDZ2N2E2IDYgMCAwIDAgMTIgMFYyWiIgLz4KPC9zdmc+Cg==) - https://lucide.dev/icons/trophy
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
function X($$renderer, $$props) {
  const $$sanitized_props = sanitize_props($$props);
  /**
   * @license lucide-svelte v0.460.1 - ISC
   *
   * This source code is licensed under the ISC license.
   * See the LICENSE file in the root directory of this source tree.
   */
  const iconNode = [
    ["path", { "d": "M18 6 6 18" }],
    ["path", { "d": "m6 6 12 12" }]
  ];
  Icon($$renderer, spread_props([
    { name: "x" },
    $$sanitized_props,
    {
      /**
       * @component @name X
       * @description Lucide SVG icon component, renders SVG Element with children.
       *
       * @preview ![img](data:image/svg+xml;base64,PHN2ZyAgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIgogIHdpZHRoPSIyNCIKICBoZWlnaHQ9IjI0IgogIHZpZXdCb3g9IjAgMCAyNCAyNCIKICBmaWxsPSJub25lIgogIHN0cm9rZT0iIzAwMCIgc3R5bGU9ImJhY2tncm91bmQtY29sb3I6ICNmZmY7IGJvcmRlci1yYWRpdXM6IDJweCIKICBzdHJva2Utd2lkdGg9IjIiCiAgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIgogIHN0cm9rZS1saW5lam9pbj0icm91bmQiCj4KICA8cGF0aCBkPSJNMTggNiA2IDE4IiAvPgogIDxwYXRoIGQ9Im02IDYgMTIgMTIiIC8+Cjwvc3ZnPgo=) - https://lucide.dev/icons/x
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
function Navbar($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const links = [
      { href: "/dashboard", label: "MY PLACE" },
      { href: "/denial", label: "DENIAL" },
      { href: "/drill", label: "DRILL" },
      { href: "/leaderboard", label: "BOARD" },
      { href: "/feed", label: "FEED" },
      { href: "/doctrine", label: "DOCTRINE" }
    ];
    $$renderer2.push(`<nav class="nav"><div class="container nav-inner"><a href="/" class="nav-brand">`);
    Flame($$renderer2, { size: 20, class: "flame" });
    $$renderer2.push(`<!----> STREAKFORGE</a> <div class="nav-links"><!--[-->`);
    const each_array = ensure_array_like(links);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let l = each_array[$$index];
      $$renderer2.push(`<a${attr("href", l.href)}${attr_class("nav-link", void 0, { "active": page.url.pathname.startsWith(l.href) })}>${escape_html(l.label)}</a>`);
    }
    $$renderer2.push(`<!--]--></div> <div class="nav-spacer"></div> `);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<a href="/login" class="btn btn-red">SIGN IN</a>`);
    }
    $$renderer2.push(`<!--]--></div></nav> <!--[-->`);
    slot($$renderer2, $$props, "default", {});
    $$renderer2.push(`<!--]-->`);
  });
}
function BottomNav($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const links = [
      { href: "/dashboard", label: "PLACE", icon: House },
      { href: "/denial", label: "DENY", icon: Ban },
      { href: "/drill", label: "DRILL", icon: Pen_line },
      { href: "/leaderboard", label: "BOARD", icon: Trophy },
      { href: "/doctrine", label: "TEXT", icon: Scroll_text }
    ];
    $$renderer2.push(`<nav class="nav-bottom" aria-label="Mobile navigation"><!--[-->`);
    const each_array = ensure_array_like(links);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let l = each_array[$$index];
      $$renderer2.push(`<a${attr("href", l.href)}${attr_class("", void 0, {
        "active": page.url.pathname === l.href || page.url.pathname.startsWith(l.href + "/")
      })}><span class="icon">`);
      l.icon($$renderer2, { size: 18 });
      $$renderer2.push(`<!----></span> ${escape_html(l.label)}</a>`);
    }
    $$renderer2.push(`<!--]--></nav>`);
  });
}
function ToastStack($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const toasts2 = getToasts();
    $$renderer2.push(`<div class="toast-stack"><!--[-->`);
    const each_array = ensure_array_like(toasts2);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let t = each_array[$$index];
      $$renderer2.push(`<div${attr_class("toast", void 0, { "toast-error": t.type === "error" })}><span>${escape_html(t.message)}</span> <button style="background:none;border:none;color:inherit;margin-left:8px;cursor:pointer;">`);
      X($$renderer2, { size: 14 });
      $$renderer2.push(`<!----></button></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
function _layout($$renderer, $$props) {
  Navbar($$renderer, {});
  $$renderer.push(`<!----> `);
  BottomNav($$renderer);
  $$renderer.push(`<!----> <main><!--[-->`);
  slot($$renderer, $$props, "default", {});
  $$renderer.push(`<!--]--></main> `);
  ToastStack($$renderer);
  $$renderer.push(`<!----> <footer class="footer"><div class="container">STREAKFORGE — EMBRACE DEFEAT. THE FUTURE IS BLACK. <span style="margin:0 8px;">·</span> <a href="/app" style="color:var(--text-dim);text-decoration:underline;text-underline-offset:3px;">Install the app</a></div></footer>`);
}
export {
  _layout as default
};
