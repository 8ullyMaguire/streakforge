function timeAgo(iso, now = /* @__PURE__ */ new Date()) {
  const then = new Date(iso).getTime();
  const diff = Math.max(0, now.getTime() - then);
  const s = Math.floor(diff / 1e3);
  if (s < 5) return "just now";
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d ago`;
  return new Date(iso).toLocaleDateString();
}
export {
  timeAgo as t
};
