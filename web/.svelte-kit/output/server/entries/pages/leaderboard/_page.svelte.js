import { i as head, c as ensure_array_like, f as attr_class, e as escape_html, d as attr, o as stringify } from "../../../chunks/index.js";
import { t as timeAgo } from "../../../chunks/api.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const tabs = [
      { id: "daily", label: "DAILY" },
      { id: "weekly", label: "WEEKLY" },
      { id: "alltime", label: "ALL-TIME" }
    ];
    let active = "daily";
    let boards = {};
    head("c59208", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Leaderboard — StreakForge</title>`);
      });
    });
    $$renderer2.push(`<div class="container" style="max-width:720px;padding-top:28px;"><h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 20px;">WHITEBOIS WHO KNOW THEIR PLACE</h1> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="tabs"><!--[-->`);
    const each_array = ensure_array_like(tabs);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let t = each_array[$$index];
      $$renderer2.push(`<button${attr_class("tab", void 0, { "active": active === t.id })}>${escape_html(t.label)}</button>`);
    }
    $$renderer2.push(`<!--]--></div> `);
    if (!boards[active]) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="skeleton" style="height:300px;"></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div class="card" style="padding:0;overflow:hidden;"><table class="board"><thead><tr><th style="width:48px;">#</th><th>WHITEBOI</th><th style="text-align:right;">POINTS</th><th style="text-align:right;">LAST</th></tr></thead><tbody><!--[-->`);
      const each_array_1 = ensure_array_like(boards[active].entries);
      for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
        let e = each_array_1[$$index_1];
        $$renderer2.push(`<tr><td${attr_class("rank", void 0, {
          "rank-top1": e.rank === 1,
          "rank-top2": e.rank === 2,
          "rank-top3": e.rank === 3
        })}>${escape_html(e.rank)}</td><td class="username"><a${attr("href", `/profile/${e.username}`)}>${escape_html(e.username)}</a> `);
        if (e.denial_count > 0) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<span class="denial-badge"${attr("title", `${stringify(e.denial_count)} denials`)}>💧${escape_html(e.denial_count)}</span>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></td><td class="count">${escape_html(e.points)} <span style="color:var(--text-dim);font-size:11px;margin-left:4px;">(${escape_html(e.waste_count)}💦 ${escape_html(e.denial_count)}💧 ${escape_html(e.affirmation_count)}✊)</span></td><td style="text-align:right;color:var(--text-dim);font-size:12px;">`);
        if (e.last_log_at) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`${escape_html(timeAgo(e.last_log_at))}`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></td></tr>`);
      }
      $$renderer2.push(`<!--]--></tbody></table> `);
      if (boards[active].entries.length === 0) {
        $$renderer2.push("<!--[0-->");
        $$renderer2.push(`<div style="padding:24px;text-align:center;color:var(--text-dim);font-size:13px;letter-spacing:0.05em;">NO LOGS IN THIS WINDOW — BE THE FIRST TO WASTE A LOAD</div>`);
      } else {
        $$renderer2.push("<!--[-1-->");
      }
      $$renderer2.push(`<!--]--></div> <p style="color:var(--text-dim);font-size:12px;margin-top:10px;text-align:center;">Denial counts 10× a waste. Three affirmations = one wasted load. Points = waste + denial×10 + affs÷3.</p>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
