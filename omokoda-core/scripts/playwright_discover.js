// playwright_discover.js — SkillForge Stage 1c discovery driver.
// Generalized from verify_vantage_ui.js: instead of asserting a known page
// shape, it drives ANY booted app and reports what it actually found —
// every network request fired while navigating + clicking, so a UI-only app
// with no OpenAPI/route-declaration surface still yields a real capability
// list instead of the empty-routes fallback.
//
// Usage: node playwright_discover.js --target http://localhost:PORT [--max-clicks 15] [--timeout-ms 20000]
// Emits one JSON object on stdout. Never throws to the caller.
const { chromium } = require('/opt/ares/node_modules/playwright');

function arg(name, def) {
  const i = process.argv.indexOf('--' + name);
  return i >= 0 ? process.argv[i + 1] : def;
}

async function main() {
  const target = arg('target');
  const maxClicks = parseInt(arg('max-clicks', '15'), 10);
  const timeoutMs = parseInt(arg('timeout-ms', '20000'), 10);
  const result = {
    ok: false,
    target,
    requests: [],       // every network request captured {method, url}
    pages_visited: [],
    clicked: [],
    rendered_text_sample: '',
    ui_only: false,
    error: null,
  };
  if (!target) {
    result.error = 'missing --target';
    console.log(JSON.stringify(result));
    return;
  }

  let browser;
  try {
    browser = await chromium.launch({ headless: true, args: ['--no-sandbox', '--disable-setuid-sandbox'] });
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    const page = await context.newPage();

    const seen = new Set();
    page.on('request', (req) => {
      const key = req.method() + ' ' + req.url();
      if (seen.has(key)) return;
      seen.add(key);
      // Only record same-origin + XHR/fetch/API-shaped calls — asset noise
      // (css/js/img/font) is not a capability surface.
      const rtype = req.resourceType();
      if (['xhr', 'fetch', 'document'].includes(rtype)) {
        result.requests.push({ method: req.method(), url: req.url(), resource_type: rtype });
      }
    });

    await page.goto(target, { waitUntil: 'networkidle', timeout: timeoutMs }).catch(() => {});
    result.pages_visited.push(target);
    result.rendered_text_sample = (await page.textContent('body').catch(() => '') || '').slice(0, 2000);

    // Click every distinct visible, enabled link/button up to maxClicks —
    // this is what surfaces JS-driven routes/forms static analysis can't see.
    const clickable = await page.$$('a[href], button, [role=button]');
    let clicks = 0;
    for (const el of clickable) {
      if (clicks >= maxClicks) break;
      try {
        const visible = await el.isVisible();
        if (!visible) continue;
        const label = (await el.textContent().catch(() => '') || '').trim().slice(0, 60);
        await el.click({ timeout: 2000 }).catch(() => {});
        await page.waitForTimeout(400);
        result.clicked.push(label || '(unlabeled)');
        clicks++;
      } catch (_) { /* keep going */ }
    }

    // ui_only: real page(s) rendered, but zero xhr/fetch calls captured —
    // means whatever this app does, it doesn't do it over a network API this
    // driver can proxy; page-agent (DOM-level control) is the right fallback.
    const apiCalls = result.requests.filter(r => r.resource_type !== 'document');
    result.ui_only = result.pages_visited.length > 0 && apiCalls.length === 0;
    result.ok = true;
  } catch (e) {
    result.error = String(e && e.message || e);
  } finally {
    if (browser) await browser.close().catch(() => {});
  }
  console.log(JSON.stringify(result));
}

main();
