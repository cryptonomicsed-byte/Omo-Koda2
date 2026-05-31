"""
Web search tool — queries DuckDuckGo Lite and returns structured results.
Tier 0: available to all agents.
"""

import re
from typing import Optional

try:
    import requests as _requests
    _HAS_REQUESTS = True
except ImportError:
    _HAS_REQUESTS = False


def web_search(params: str) -> str:
    """
    Search the web via DuckDuckGo Lite.

    Params: plain query string, or JSON {"query": "...", "max_results": 5}
    """
    import json

    query = params.strip()
    max_results = 5

    if query.startswith("{"):
        try:
            obj = json.loads(query)
            query = obj.get("query", query)
            max_results = int(obj.get("max_results", max_results))
        except (json.JSONDecodeError, ValueError):
            pass

    if not query:
        raise ValueError("web_search requires a non-empty query")

    if not _HAS_REQUESTS:
        return f"[web_search stub — requests not installed] Query: {query}"

    url = "https://duckduckgo.com/lite/"
    resp = _requests.get(
        url,
        params={"q": query},
        headers={"User-Agent": "Mozilla/5.0 (compatible; OmoKoda/1.0)"},
        timeout=10,
    )
    resp.raise_for_status()

    results = _parse_ddg_lite(resp.text, max_results)
    if not results:
        return f"No results found for: {query}"

    lines = [f"Search results for: {query}\n"]
    for i, r in enumerate(results, 1):
        lines.append(f"{i}. {r['title']}")
        if r.get("url"):
            lines.append(f"   {r['url']}")
        if r.get("snippet"):
            lines.append(f"   {r['snippet']}")
        lines.append("")

    return "\n".join(lines).strip()


def _parse_ddg_lite(html: str, max_results: int) -> list[dict]:
    results: list[dict] = []

    # DuckDuckGo Lite result links are in <a class="result-link">
    link_pattern = re.compile(
        r'<a[^>]+class="result-link"[^>]*href="([^"]+)"[^>]*>(.*?)</a>',
        re.DOTALL,
    )
    snippet_pattern = re.compile(
        r'<td class="result-snippet"[^>]*>(.*?)</td>',
        re.DOTALL,
    )

    links = link_pattern.findall(html)
    snippets = [m.group(1) for m in snippet_pattern.finditer(html)]

    for i, (url, title) in enumerate(links[:max_results]):
        title = re.sub(r"<[^>]+>", "", title).strip()
        snippet = re.sub(r"<[^>]+>", "", snippets[i]).strip() if i < len(snippets) else ""
        results.append({"title": title, "url": url, "snippet": snippet})

    return results
