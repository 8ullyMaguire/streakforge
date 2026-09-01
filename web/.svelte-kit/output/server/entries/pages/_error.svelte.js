import { e as escape_html } from "../../chunks/index.js";
import "clsx";
import { p as page } from "../../chunks/index2.js";
function _error($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="container" style="text-align:center;padding:80px 20px;"><h1 style="font-size:64px;margin:0;">${escape_html(page.status)}</h1> <p style="color:var(--text-dim);">Something went wrong on our forge.</p> <a href="/" class="btn btn-red" style="margin-top:16px;">BACK TO HALL</a></div>`);
  });
}
export {
  _error as default
};
