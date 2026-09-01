import { c as ensure_array_like, e as escape_html, h as derived, i as head, d as attr } from "../../chunks/index.js";
import { t as timeAgo } from "../../chunks/api.js";
function Marquee($$renderer, $$props) {
  let { items } = $$props;
  let groups = derived(() => [items, items]);
  $$renderer.push(`<div class="marquee" aria-hidden="true"><div class="marquee-track"><!--[-->`);
  const each_array = ensure_array_like(groups());
  for (let gi = 0, $$length = each_array.length; gi < $$length; gi++) {
    let group = each_array[gi];
    $$renderer.push(`<div class="marquee-group"><!--[-->`);
    const each_array_1 = ensure_array_like(group);
    for (let i = 0, $$length2 = each_array_1.length; i < $$length2; i++) {
      let item = each_array_1[i];
      $$renderer.push(`<span class="dot"></span> <span>${escape_html(item.username)}</span> <span style="color:var(--text-dim);">${escape_html(item.time)}</span>`);
    }
    $$renderer.push(`<!--]--></div>`);
  }
  $$renderer.push(`<!--]--></div></div>`);
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let recentFeed = [];
    head("1uha8ag", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>StreakForge — Embrace Defeat. The Future Is Black.</title>`);
      });
    });
    $$renderer2.push(`<section class="hero"><div class="tagline">EMBRACE DEFEAT. THE FUTURE IS BLACK.</div> <h1>StreakForge</h1> <p class="sub">Whitebois don't just track habits. They deny. They lock. They waste when permitted.
		And the board remembers who knows their place.</p> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="cta-row">`);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<a href="/login" class="btn btn-red btn-lg">KNOW YOUR PLACE</a>`);
    }
    $$renderer2.push(`<!--]--> <a href="/leaderboard" class="btn btn-ghost btn-lg">VIEW THE BOARD</a></div> <p style="color:var(--text-dim);font-size:13px;margin-top:14px;">`);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`Create an account. Own your submission. Deny harder.`);
    }
    $$renderer2.push(`<!--]--></p></section> `);
    if (recentFeed.length > 0) {
      $$renderer2.push("<!--[0-->");
      Marquee($$renderer2, {
        items: recentFeed.slice(0, 10).map((i) => ({ username: i.username, time: timeAgo(i.logged_at) }))
      });
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="container">`);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (recentFeed.length > 0) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<h2 style="font-size:18px;letter-spacing:0.08em;margin:28px 0 12px;">CONFESSION FEED</h2> <div class="card" style="padding:4px 18px;"><!--[-->`);
      const each_array_3 = ensure_array_like(recentFeed);
      for (let $$index_3 = 0, $$length = each_array_3.length; $$index_3 < $$length; $$index_3++) {
        let item = each_array_3[$$index_3];
        $$renderer2.push(`<div class="feed-item"><div><span class="username" style="font-weight:600;"><a${attr("href", `/profile/${item.username}`)}>${escape_html(item.username)}</a></span> `);
        if (item.note) {
          $$renderer2.push("<!--[0-->");
          $$renderer2.push(`<div class="note">“${escape_html(item.note)}”</div>`);
        } else {
          $$renderer2.push("<!--[-1-->");
        }
        $$renderer2.push(`<!--]--></div> <div style="flex:1;"></div> <span class="time">${escape_html(timeAgo(item.logged_at))}</span></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--></div> `);
    {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="container"><div class="skeleton" style="height:220px;margin-top:24px;"></div></div>`);
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  _page as default
};
