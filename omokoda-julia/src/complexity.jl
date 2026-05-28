# BBU (Busy Beaver Unit) complexity scoring.
#
# Maps code complexity onto the Busy Beaver scale:
#   1.0   → trivial (BB(1) = 1)
#   6.0   → simple  (BB(2) = 6)
#   21.0  → moderate (BB(3) = 21)
#   107.0 → complex  (BB(4) = 107)
#   47.1  → exceptional (maps symbolically to BB(5) = 47_176_870, scaled to 47.1)
#
# Scoring factors:
#   - Token count (raw complexity signal)
#   - Nesting depth (exponential branching)
#   - Conditional branches (if/else/match/switch/case/when/unless)
#   - Loop constructs (for/while/loop/do/repeat/until/foreach/map/reduce/filter)

"""
    count_tokens(text::String) -> Int

Count whitespace-delimited tokens in `text`.
"""
function count_tokens(text::String)::Int
    tokens = split(strip(text), r"\s+")
    # split on empty string returns [""], so filter
    return count(t -> !isempty(t), tokens)
end

"""
    measure_nesting_depth(text::String) -> Int

Estimate maximum nesting depth by counting the maximum open brace/bracket/paren depth
plus indentation-based block depth (for languages using indentation).
"""
function measure_nesting_depth(text::String)::Int
    max_depth = 0
    current_depth = 0
    for ch in text
        if ch in ('{', '[', '(')
            current_depth += 1
            if current_depth > max_depth
                max_depth = current_depth
            end
        elseif ch in ('}', ']', ')')
            if current_depth > 0
                current_depth -= 1
            end
        end
    end

    # Also check indentation depth (for Python, Julia, Ruby, etc.)
    indent_depth = 0
    max_indent = 0
    for line in split(text, '\n')
        # Count leading spaces (4 spaces = 1 level, tabs = 1 level each)
        leading = length(line) - length(lstrip(line))
        spaces = count(c -> c == ' ', line[1:min(leading, length(line))])
        tabs   = count(c -> c == '\t', line[1:min(leading, length(line))])
        indent_depth = div(spaces, 4) + tabs
        if indent_depth > max_indent
            max_indent = indent_depth
        end
    end

    return max(max_depth, max_indent)
end

"""
    count_conditionals(text::String) -> Int

Count conditional branch keywords across multiple programming languages.
"""
function count_conditionals(text::String)::Int
    # Case-insensitive match for whole words
    patterns = [
        r"\bif\b", r"\belse\b", r"\belseif\b", r"\belif\b",
        r"\bmatch\b", r"\bswitch\b", r"\bcase\b", r"\bwhen\b",
        r"\bunless\b", r"\bternary\b", r"\b\?\s*:", r"\bcond\b",
    ]
    count_total = 0
    lower_text = lowercase(text)
    for pat in patterns
        count_total += length(collect(eachmatch(pat, lower_text)))
    end
    return count_total
end

"""
    count_loops(text::String) -> Int

Count loop constructs across multiple programming languages.
"""
function count_loops(text::String)::Int
    patterns = [
        r"\bfor\b", r"\bwhile\b", r"\bloop\b", r"\bdo\b",
        r"\brepeat\b", r"\buntil\b", r"\bforeach\b",
        r"\bmap\b", r"\breduce\b", r"\bfilter\b",
        r"\bfold\b", r"\biter\b", r"\beach\b",
    ]
    count_total = 0
    lower_text = lowercase(text)
    for pat in patterns
        count_total += length(collect(eachmatch(pat, lower_text)))
    end
    return count_total
end

"""
    calculate_bbu(code_text::String) -> Float64

Calculate a BBU (Busy Beaver Unit) complexity score for `code_text`.

Returns a value in [1.0, 47.1] where:
  - 1.0  = trivial (empty or near-empty code)
  - 6.0  = simple
  - 21.0 = moderate
  - 107.0 would be complex, but score is capped at 47.1
  - 47.1 = exceptional complexity (symbolic BB(5) reference)

The raw score is computed as a weighted sum of complexity factors,
then mapped onto the BBU scale via sigmoid-like normalization.
"""
function calculate_bbu(code_text::String)::Float64
    text = strip(code_text)

    if isempty(text)
        return 1.0
    end

    tokens      = count_tokens(text)
    nesting     = measure_nesting_depth(text)
    conditionals = count_conditionals(text)
    loops       = count_loops(text)

    # Raw complexity score (weighted sum)
    # Weights tuned so that:
    #   - 0 tokens → raw ≈ 0 → BBU = 1.0
    #   - ~5 tokens, no structure → raw ≈ 5 → BBU ≈ 6.0
    #   - ~20 tokens with mild nesting → raw ≈ 20 → BBU ≈ 21.0
    #   - heavily nested with many branches → raw high → BBU → 47.1
    raw = (tokens       * 0.15) +
          (nesting      * 4.0)  +
          (conditionals * 2.5)  +
          (loops        * 3.0)

    # Map raw score to BBU scale using a piecewise approach aligned to BB values.
    # BB thresholds: 1, 6, 21, 47.1 (capped)
    # Raw thresholds (approximate): 0, 5, 20, 80+
    bbu = if raw <= 0.0
        1.0
    elseif raw <= 5.0
        1.0 + (raw / 5.0) * (6.0 - 1.0)
    elseif raw <= 20.0
        6.0 + ((raw - 5.0) / 15.0) * (21.0 - 6.0)
    elseif raw <= 80.0
        21.0 + ((raw - 20.0) / 60.0) * (47.1 - 21.0)
    else
        47.1
    end

    # Clamp to [1.0, 47.1]
    return clamp(bbu, 1.0, 47.1)
end
