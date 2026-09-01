import { i as head, c as ensure_array_like, f as attr_class, e as escape_html } from "../../../chunks/index.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let docs = [];
    let activeId = null;
    head("128xxqr", $$renderer2, ($$renderer3) => {
      $$renderer3.title(($$renderer4) => {
        $$renderer4.push(`<title>Doctrine — StreakForge</title>`);
      });
    });
    $$renderer2.push(`<div class="container" style="padding-top:28px;"><h1 style="font-size:24px;letter-spacing:0.04em;margin:0 0 4px;">THE DOCTRINE</h1> <p style="color:var(--text-dim);font-size:14px;margin:0 0 20px;">Doctrine. Commandments. Training. The collected texts.</p> `);
    {
      $$renderer2.push("<!--[-1-->");
      $$renderer2.push(`<div style="display:flex;gap:20px;flex-wrap:wrap;align-items:flex-start;"><aside style="flex:0 0 240px;min-width:200px;position:sticky;top:76px;max-height:calc(100vh - 100px);overflow-y:auto;"><div class="card" style="padding:8px;"><!--[-->`);
      const each_array = ensure_array_like(docs);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let d = each_array[$$index];
        $$renderer2.push(`<button${attr_class("doc-link svelte-128xxqr", void 0, { "active": activeId === d.id })} style="display:block;width:100%;text-align:left;padding:10px 12px;border:none;background:transparent;color:var(--text-dim);font-size:14px;border-radius:8px;cursor:pointer;transition:background .15s,color .15s;">${escape_html(d.title)}</button>`);
      }
      $$renderer2.push(`<!--]--></div></aside> <div style="flex:1;min-width:280px;">`);
      {
        $$renderer2.push("<!--[-1-->");
        $$renderer2.push(`<div class="card empty"><div class="big">📜</div> Select a text from the list.</div>`);
      }
      $$renderer2.push(`<!--]--></div></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
