package main

// ecosystemUIHTML — minimal human-readable page over the same live data the
// agent-facing /ecosystem and /v1/token/{address} endpoints return. No build
// step, no external assets: inline CSS + vanilla JS only.
//
// NOTE: the JS must not use backtick template literals — the whole page is a
// Go raw string.

const ecoUIHTML = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ọmọ Kọ́dà · Ecosystem Surface</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  body { margin: 0; background: #0d1117; color: #c9d1d9;
         font: 14px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
  header { padding: 18px 24px; border-bottom: 1px solid #21262d;
           display: flex; align-items: baseline; gap: 14px; flex-wrap: wrap; }
  header h1 { font-size: 17px; margin: 0; color: #e6edf3; letter-spacing: .5px; }
  header .ts { color: #8b949e; font-size: 12px; }
  main { padding: 18px 24px; max-width: 1100px; margin: 0 auto; }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 14px; }
  section { background: #161b22; border: 1px solid #21262d; border-radius: 8px;
            padding: 14px 16px; }
  section h2 { margin: 0 0 10px; font-size: 12px; text-transform: uppercase;
               letter-spacing: 1.2px; color: #58a6ff; }
  .k { color: #8b949e; }
  .v { color: #e6edf3; }
  .badge { display: inline-block; padding: 1px 8px; border-radius: 10px;
           font-size: 11px; font-weight: 600; }
  .b-up { background: #1f6feb33; color: #58a6ff; }
  .b-ok { background: #23863633; color: #3fb950; }
  .b-warn { background: #9e6a0333; color: #d29922; }
  .b-err { background: #da363333; color: #f85149; }
  table { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  th, td { text-align: left; padding: 5px 8px; border-bottom: 1px solid #21262d;
           white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 220px; }
  th { color: #8b949e; font-weight: 600; }
  .err { color: #f85149; font-size: 12px; }
  form { display: flex; gap: 8px; margin: 4px 0 16px; }
  input { flex: 1; background: #0d1117; border: 1px solid #30363d; color: #e6edf3;
          border-radius: 6px; padding: 8px 10px; font: inherit; }
  button { background: #21262d; border: 1px solid #30363d; color: #e6edf3;
           border-radius: 6px; padding: 8px 14px; font: inherit; cursor: pointer; }
  button:hover { background: #30363d; }
  pre { background: #0d1117; border: 1px solid #21262d; border-radius: 6px;
        padding: 10px; overflow: auto; font-size: 11.5px; max-height: 420px; }
  .muted { color: #8b949e; }
</style>
</head>
<body>
<header>
  <h1>Ọmọ Kọ́dà · Ecosystem Surface</h1>
  <span class="ts" id="ts"></span>
  <span class="muted">council · picks · pool health · wallet intel</span>
</header>
<main>
  <form id="lookup">
    <input id="addr" placeholder="token address (solana mint) — e.g. 7c9sXHS879M9fq9Gex4pCSHYGeBDVK7yErKGMAxypump"
           autocomplete="off" spellcheck="false">
    <button type="submit">ask Omokoda</button>
  </form>
  <div id="out"></div>
  <div class="grid" id="grid"></div>
</main>
<script>
var esc = function (s) {
  return String(s == null ? "" : s).replace(/[&<>"']/g, function (c) {
    return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c];
  });
};
var el = function (tag, cls, html) {
  var e = document.createElement(tag);
  if (cls) e.className = cls;
  if (html != null) e.innerHTML = html;
  return e;
};
var fmtNum = function (n) {
  if (n == null) return "—";
  return Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 });
};
function section(title, inner) {
  var s = el("section");
  s.appendChild(el("h2", null, esc(title)));
  s.appendChild(el("div", null, inner));
  return s;
}
function renderEco(d) {
  document.getElementById("ts").textContent = "ts " + (d.ts || "");
  var grid = document.getElementById("grid");
  grid.innerHTML = "";
  var errs = d.source_errors || {};
  var o = d.council_overview;
  var html;
  if (o) {
    var verdicts = Array.isArray(d.council_verdicts) ? d.council_verdicts : [];
    html = "<p><span class=\"k\">daemon</span> <span class=\"v\">" +
      (o.daemon_running ? "running" : "down") + "</span> · " +
      "<span class=\"k\">verdicts</span> <span class=\"v\">" + fmtNum(o.verdict_count) + "</span> · " +
      "<span class=\"k\">pending</span> <span class=\"v\">" + fmtNum(o.trace_buffer_pending) + "</span></p>";
    html += "<table><tr><th>symbol</th><th>dir</th><th>conv</th><th>outcome</th></tr>";
    for (var i = 0; i < Math.min(8, verdicts.length); i++) {
      var v = verdicts[i];
      var cls = v.direction === "BUY" ? "b-up" : (v.direction === "SELL" ? "b-err" : "b-warn");
      html += "<tr><td>" + esc(v.symbol) + "</td><td><span class=\"badge " + cls + "\">" +
        esc(v.direction) + "</span></td><td>" + esc(v.conviction) + "</td><td>" +
        esc(v.outcome) + "</td></tr>";
    }
    html += "</table>";
  } else {
    html = "<span class=\"err\">unavailable: " + esc(errs.council_overview || errs.council_verdicts) + "</span>";
  }
  grid.appendChild(section("Council", html));

  if (d.picks && Array.isArray(d.picks.picks)) {
    html = "<table><tr><th>#</th><th>symbol</th><th>score</th><th>status</th><th>addr</th></tr>";
    for (var j = 0; j < d.picks.picks.length; j++) {
      var p = d.picks.picks[j];
      html += "<tr><td>" + esc(p.rank) + "</td><td>" + esc(p.symbol) + "</td><td>" +
        esc(p.score) + "</td><td>" + esc(p.status) + "</td><td title=\"" +
        esc(p.token_addr) + "\">" + esc(p.token_addr) + "</td></tr>";
    }
    html += "</table>";
  } else {
    html = "<span class=\"err\">unavailable: " + esc(errs.picks) + "</span>";
  }
  grid.appendChild(section("Top picks", html));

  var ph = d.pool_health;
  if (ph) {
    var g = ph.gmgn || {}, px = ph.proxies || {}, sl = ph.solscan || {}, tm = ph.teamorouter || {};
    html = "<p><span class=\"k\">gmgn keys</span> <span class=\"v\">" + fmtNum(g.keys) + "</span>" +
      " · <span class=\"k\">cooling</span> <span class=\"v\">" + fmtNum(g.cooling) + "</span>" +
      (g.ip_banned ? " · <span class=\"badge b-err\">IP BANNED</span>" : "") + "</p>" +
      "<p><span class=\"k\">proxies</span> <span class=\"v\">" + fmtNum(px.total) + "</span>" +
      " · <span class=\"k\">live est</span> <span class=\"v\">" + fmtNum(px.live_estimate) + "</span>" +
      " · <span class=\"k\">cooldown</span> <span class=\"v\">" + fmtNum(px.cooldown) + "</span></p>" +
      "<p><span class=\"k\">solscan keys</span> <span class=\"v\">" + fmtNum(sl.keys) + "</span>" +
      " · <span class=\"k\">teamorouter keys</span> <span class=\"v\">" + fmtNum(tm.keys) + "</span></p>";
  } else {
    html = "<span class=\"err\">unavailable: " + esc(errs.pool_health) + "</span>";
  }
  grid.appendChild(section("Pool health", html));

  var wi = d.wallet_intel;
  if (wi && wi.stats) {
    var st = wi.stats;
    html = "<p><span class=\"k\">wallets</span> <span class=\"v\">" + fmtNum(st.wallets) + "</span>" +
      " · <span class=\"k\">tokens</span> <span class=\"v\">" + fmtNum(st.tokens) + "</span>" +
      " · <span class=\"k\">seen txs</span> <span class=\"v\">" + fmtNum(st.seen_txs) + "</span></p>";
    html += "<table><tr><th>token</th><th>buy vol</th><th>wallets</th></tr>";
    var tops = (wi.top_tokens || []).slice(0, 5);
    for (var k = 0; k < tops.length; k++) {
      html += "<tr><td>" + esc(tops[k].symbol) + "</td><td>$" + fmtNum(tops[k].buy_volume) +
        "</td><td>" + fmtNum(tops[k].distinct_wallets) + "</td></tr>";
    }
    html += "</table>";
  } else {
    html = "<span class=\"err\">unavailable: " + esc(errs.wallet_intel) + "</span>";
  }
  grid.appendChild(section("Wallet intel", html));
}
function renderToken(d) {
  var out = document.getElementById("out");
  out.innerHTML = "";
  var vs = (d.council_verdicts || []).map(function (v) {
    return { id: v.id, symbol: v.symbol, direction: v.direction,
             conviction: v.conviction, outcome: v.outcome };
  });
  var compact = {
    query: d.query, found: d.found, picks_score: d.picks_score,
    council_verdicts: vs, wallet_intel: d.wallet_intel,
    pool_health: d.pool_health, source_errors: d.source_errors || {}
  };
  out.appendChild(el("pre", null, JSON.stringify(compact, null, 2)));
}
function boot() {
  fetch("/ecosystem").then(function (r) { return r.json(); }).then(renderEco).catch(function (e) {
    document.getElementById("grid").appendChild(
      el("section", null, "<h2>Ecosystem</h2><span class=\"err\">failed: " + esc(e.message) + "</span>"));
  });
  document.getElementById("lookup").addEventListener("submit", function (ev) {
    ev.preventDefault();
    var addr = document.getElementById("addr").value.trim();
    if (!addr) return;
    fetch("/v1/token/" + encodeURIComponent(addr)).then(function (r) { return r.json(); })
      .then(renderToken).catch(function (e) {
        document.getElementById("out").appendChild(
          el("pre", null, "<span class=\"err\">failed: " + esc(e.message) + "</span>"));
      });
  });
}
boot();
</script>
</body>
</html>`
