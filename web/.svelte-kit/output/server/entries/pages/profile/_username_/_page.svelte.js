import { i as head, e as escape_html } from "../../../../chunks/index.js";
import "../../../../chunks/client.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    head("y31r23", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>${escape_html("Profile")} — StreakForge</title>`);
      });
    });
    $$renderer2.push(`<div class="container" style="max-width:640px;padding-top:28px;">`);
    {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="skeleton" style="height:280px;"></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
