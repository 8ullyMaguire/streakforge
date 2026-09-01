import { i as head, c as ensure_array_like, d as attr, e as escape_html } from "../../../chunks/index.js";
import { t as timeAgo } from "../../../chunks/api.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let items = [];
    let loading = false;
    head("1ooj66h", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Feed — StreakForge</title>`);
      });
    });
    $$renderer2.push(`<div class="container" style="max-width:640px;padding-top:28px;"><h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 20px;">PUBLIC ACTIVITY FEED</h1> <div class="card" style="padding:4px 18px;">`);
    if (items.length === 0 && !loading) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="empty"><div class="big">🕳️</div> No completions yet — be the first.</div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<!--[-->`);
      const each_array = ensure_array_like(items);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let item = each_array[$$index];
        $$renderer2.push(`<div class="feed-item"><img class="avatar avatar-md"${attr("src", item.avatar_url ?? `https://api.dicebear.com/7.x/identicon/svg?seed=${item.username}`)} alt=""/> <div style="flex:1;"><span class="username" style="font-weight:600;"><a${attr("href", `/profile/${item.username}`)}>${escape_html(item.username)}</a></span> <span style="color:var(--text-dim);font-size:12px;">logged a completion</span> `);
        if (item.note) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<div class="note">“${escape_html(item.note)}”</div>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></div> <span class="time">${escape_html(timeAgo(item.logged_at))}</span></div>`);
      }
      $$renderer2.push(`<!--]-->`);
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div> `);
    if (items.length > 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div style="text-align:center;margin:20px 0;"><button class="btn btn-ghost"${attr("disabled", loading, true)}>${escape_html("LOAD MORE")}</button></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
