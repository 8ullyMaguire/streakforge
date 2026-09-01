import { i as head, f as attr_class, d as attr, e as escape_html } from "../../../chunks/index.js";
new TextEncoder();
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let mode = "login";
    let username = "";
    let password = "";
    let website = "";
    let busy = false;
    head("1x05zx6", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Sign in — StreakForge</title>`);
      });
    });
    $$renderer2.push(`<div class="container"><div class="card auth-card"><h1 style="font-size:26px;letter-spacing:0.04em;margin:0 0 8px;">SIGN IN</h1> <p style="color:var(--text-dim);font-size:14px;margin:0 0 24px;">Own your submission. No external accounts needed.</p> <div class="auth-tabs" role="tablist"><button${attr_class("auth-tab", void 0, { "active": mode === "login" })} role="tab"${attr("aria-selected", mode === "login")}>SIGN IN</button> <button${attr_class("auth-tab", void 0, { "active": mode === "register" })} role="tab"${attr("aria-selected", mode === "register")}>REGISTER</button></div> <form><div class="form-row"><label for="username">Username</label> <input id="username" name="username"${attr("value", username)} autocomplete="username" maxlength="30" spellcheck="false" required=""/></div> <div class="form-row"><label for="password">Password</label> <input id="password" name="password" type="password"${attr("value", password)}${attr("autocomplete", "current-password")} minlength="8" required=""/></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="honeypot" aria-hidden="true"><label for="website">Website</label> <input id="website" name="website" type="text" tabindex="-1" autocomplete="off"${attr("value", website)}/></div> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <button class="btn btn-red btn-block" type="submit"${attr("disabled", busy, true)}>${escape_html("SIGN IN")}</button></form> <p style="color:var(--text-dim);font-size:12px;margin-top:18px;">${escape_html("New here? Switch to REGISTER to claim your username.")}</p></div></div>`);
  });
}
export {
  _page as default
};
