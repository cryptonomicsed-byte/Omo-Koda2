"""
Data analysis tool — statistical analysis over JSON/CSV payloads.
Tier 1 (Builder): available once an agent has earned basic trust.

Uses only Python stdlib (statistics, csv, json) — no pandas/numpy required.
"""

import csv
import io
import json
import statistics
from typing import Any


def data_analysis(params: str) -> str:
    """
    Analyze numeric data and return descriptive statistics.

    Params JSON:
      { "data": [1, 2, 3, ...], "format": "json" }         — flat numeric array
      { "data": "col1,col2\\n1,2\\n3,4", "format": "csv", "column": "col1" }
      { "operation": "summary" | "histogram" | "correlate", ... }
    """
    obj = _parse_params(params)

    operation = obj.get("operation", "summary")
    fmt = obj.get("format", "json")
    data_raw = obj.get("data", [])

    numbers = _load_numbers(data_raw, fmt, obj.get("column"))

    if not numbers:
        raise ValueError("no numeric data found in input")

    if operation == "summary":
        return json.dumps(_summary(numbers), indent=2)
    elif operation == "histogram":
        bins = int(obj.get("bins", 10))
        return json.dumps(_histogram(numbers, bins), indent=2)
    elif operation == "correlate":
        second_raw = obj.get("data2", [])
        second_col = obj.get("column2")
        numbers2 = _load_numbers(second_raw, fmt, second_col)
        return json.dumps(_correlate(numbers, numbers2), indent=2)
    else:
        raise ValueError(f"unknown operation: {operation}. Use: summary, histogram, correlate")


def _parse_params(params: str) -> dict:
    params = params.strip()
    if params.startswith("{"):
        return json.loads(params)
    # Bare JSON array
    if params.startswith("["):
        return {"data": json.loads(params)}
    raise ValueError("data_analysis params must be JSON object or array")


def _load_numbers(data: Any, fmt: str, column: str | None) -> list[float]:
    if isinstance(data, list):
        return [float(x) for x in data if _is_numeric(x)]

    if isinstance(data, str):
        if fmt == "csv":
            reader = csv.DictReader(io.StringIO(data))
            rows = list(reader)
            if column:
                return [float(r[column]) for r in rows if _is_numeric(r.get(column, ""))]
            # Try first numeric column
            if rows:
                for col in rows[0]:
                    try:
                        return [float(r[col]) for r in rows]
                    except (ValueError, TypeError):
                        continue
        else:
            parsed = json.loads(data)
            return _load_numbers(parsed, fmt, column)

    return []


def _is_numeric(v: Any) -> bool:
    try:
        float(v)
        return True
    except (ValueError, TypeError):
        return False


def _summary(nums: list[float]) -> dict:
    n = len(nums)
    sorted_nums = sorted(nums)
    result: dict = {
        "count": n,
        "mean": round(statistics.mean(nums), 6),
        "median": round(statistics.median(nums), 6),
        "min": min(nums),
        "max": max(nums),
        "range": max(nums) - min(nums),
    }
    if n >= 2:
        result["stdev"] = round(statistics.stdev(nums), 6)
        result["variance"] = round(statistics.variance(nums), 6)
    if n >= 4:
        q1 = sorted_nums[n // 4]
        q3 = sorted_nums[(3 * n) // 4]
        result["q1"] = q1
        result["q3"] = q3
        result["iqr"] = q3 - q1
    return result


def _histogram(nums: list[float], bins: int) -> dict:
    lo, hi = min(nums), max(nums)
    if lo == hi:
        return {"bins": [{"range": [lo, hi], "count": len(nums)}]}

    width = (hi - lo) / bins
    counts = [0] * bins
    for v in nums:
        idx = min(int((v - lo) / width), bins - 1)
        counts[idx] += 1

    return {
        "bins": [
            {
                "range": [round(lo + i * width, 6), round(lo + (i + 1) * width, 6)],
                "count": counts[i],
            }
            for i in range(bins)
        ]
    }


def _correlate(x: list[float], y: list[float]) -> dict:
    n = min(len(x), len(y))
    if n < 2:
        raise ValueError("need at least 2 paired values for correlation")
    x, y = x[:n], y[:n]
    try:
        r = statistics.correlation(x, y)
    except AttributeError:
        # statistics.correlation added in Python 3.10; fallback
        mx, my = statistics.mean(x), statistics.mean(y)
        num = sum((xi - mx) * (yi - my) for xi, yi in zip(x, y))
        den = (sum((xi - mx) ** 2 for xi in x) * sum((yi - my) ** 2 for yi in y)) ** 0.5
        r = num / den if den else 0.0
    return {"n": n, "pearson_r": round(r, 6)}
