# RsCel Usage Guide

RsCel implements the [Common Expression Language (CEL)](https://github.com/google/cel-spec) in Rust. This guide describes the expression syntax exposed by `rscel` and the default macros, type constructors, and functions that are preloaded when you build a `BindContext` with `BindContext::new()`.

## Running an Expression

```rust
use rscel::{BindContext, CelContext, CelValue};

fn main() -> rscel::CelResult<()> {
    let mut programs = CelContext::new();
    let mut bindings = BindContext::new();

    programs.add_program_str("main", "greeting + ' ' + subject")?;
    bindings.bind_param("greeting", "hello".into());
    bindings.bind_param("subject", "world".into());

    let value = programs.exec("main", &bindings)?;
    assert_eq!(value, "hello world".into());
    Ok(())
}
```

`BindContext::new()` registers the macros and functions listed below and exposes CEL types ("int", "string", etc.) so that they can be compared or invoked as constructors inside an expression.

## Language Basics

- **Literals**: signed integers (`123`), unsigned integers (`123u`), floating point numbers (`3.14`, `.5`, `1.`), quoted strings (`"foo"` or `'foo'` with `\u`/`\x` escapes), byte strings (`b"abc"`), booleans, and `null`.
- **Lists**: `[expr1, expr2, ...]`. Indexing uses zero-based integers; negative indices are allowed when compiled with the `neg_index` feature.
- **Maps/Objects**: `{ 'key': value, other_key: value }`. Keys must resolve to strings at runtime.
- **Access**: `obj.field` looks up a field or method; `value[index]` indexes lists, strings, bytes, or maps.
- **Operators**: arithmetic (`+ - * / %`), comparison (`< <= > >= == !=`), logical (`!`, `&&`, `||` with short-circuit semantics), and membership (`lhs in rhs`). String membership checks substring containment; map membership checks for a key.
- **Conditionals**: `condition ? when_true : when_false`.
- **Match expressions**: `match value { case x < 0: ..., case type(string): ..., case _: ... }` supports comparison patterns, type tests (`int`, `uint`, `float`, `string`, `bool`, `bytes`, `list`, `object`, `null`, `timestamp`, `duration`), and a wildcard case.
- **Format strings**: `f"hello {name}!"` interpolates expressions that must evaluate to strings.
- **Truthiness**: numbers are truthy when non-zero, collections when non-empty, timestamps/durations/types always truthy, and `null`/errors are falsy. Logical operators and macros rely on this notion.
- **Errors**: runtime errors propagate as special values; most helpers short-circuit when they encounter `CelError` instances.

## Default Macros (`default_macros.rs`)

Macros operate on unresolved bytecode and therefore require identifiers for loop variables. All macros are available both at compile and runtime.

| Macro | Signature | Description |
| --- | --- | --- |
| `has(expr)` | `has(expr)` | Evaluates the expression and returns `true` unless a binding or attribute error occurs; other errors propagate. |
| `coalesce(expr...)` | `coalesce(e1, e2, ...)` | Returns the first argument that does not resolve to `null` and is not a binding/attribute error; resolves each expression in order. |
| `list.all(var, predicate)` | `[1,2,3].all(x, x < 5)` | Binds each element to `var` and ensures every iteration is truthy. Returns `false` upon the first falsy result. |
| `list.exists(var, predicate)` | `[items].exists(x, test)` | Returns `true` as soon as one predicate evaluates truthy. |
| `list.exists_one(var, predicate)` | `[items].exists_one(x, test)` | Requires exactly one truthy evaluation; returns `false` if zero or more than one match. |
| `list.filter(var, predicate)` | `[items].filter(x, keep?)` | Builds a list of elements whose predicate is truthy. When invoked on a map, the identifier receives each key and returns the list of kept keys. |
| `list.map(var, mapper)` | `[items].map(x, expr)` | Collects the mapper result for every element. A ternary form `[items].map(x, predicate, mapper)` first evaluates `predicate` and only maps elements where it is truthy. With maps, the variable receives each key and returns a list of mapped values. |
| `list.reduce(acc, item, step, initial)` | `[items].reduce(curr, next, step_expr, seed)` | Initializes `curr` with `seed`; for each element binds `next` to the element, `curr` to the running total, evaluates `step_expr`, and stores the result back in `curr`. Returns the final accumulator. |
| `list.count(var, predicate)` | `[items].count(x, test)` | Returns the number of elements for which `predicate` evaluates truthy. |
| `list.find(var, predicate)` | `[items].find(x, test)` | Returns the first element for which `predicate` is truthy, or `null` if none match. |
| `list.flatMap(var, mapper)` | `[items].flatMap(x, expr)` | Maps each element through `mapper`; list results are flattened one level into the output. Non-list results are appended directly. |

## Type Constructors (`type_funcs.rs`)

These helpers can be invoked either as global functions (`int(value)`) or as methods (`value.int()` when dispatched that way) and mirror CEL's built-in type conversion rules.

| Function | Accepted Inputs | Output / Notes |
| --- | --- | --- |
| `bool(x)` | `bool`, supported strings (`"1"`, `"0"`, `"t"`, `"f"`, case-insensitive `"true"`/`"false"`), and under the optional `type_prop` feature any value | Returns a boolean or raises `value()` on invalid strings. |
| `int(x)` | `int`, `uint`, `float`, `bool`, `string` (parsed as base-10), `timestamp` | Converts to `i64`; parsing failures raise `value()` errors. |
| `uint(x)` | `uint`, `int`, `float`, `bool`, `string` | Converts to `u64`; negative numbers or invalid strings raise errors. |
| `double(x)` / `float(x)` | `float`, `int`, `uint`, `bool`, `string` | Produces an `f64`; parsing errors surface as `value()` errors. |
| `string(x)` | numbers, strings, UTF-8 bytes, timestamps (RFC3339), durations, others | Converts to string; non UTF-8 bytes produce an error. |
| `bytes(x)` | strings, existing bytes | Returns a byte array (`CelBytes`). |
| `timestamp()` | no args, RFC3339 / RFC2822 / `DateTime<Utc>` / epoch seconds (`int`/`uint`) | Returns a UTC timestamp; invalid formats raise errors. |
| `duration(x)` | ISO-like duration strings understood by `duration_str`, integer seconds, `(seconds, nanos)` pair, `chrono::Duration` | Returns a `Duration`; invalid formats raise errors. |
| `dyn(x)` | any value | Identity; exposes the dynamic value type. |
| `type(x)` | any value | Returns the CEL type descriptor (e.g., `int`, `string`, `timestamp`). |

The constructor names (`int`, `uint`, `double`, `bytes`, etc.) are also exported as type values in the global scope, allowing comparisons such as `type(value) == int`.

## Default Functions (`default_funcs.rs`)

All functions can be called as free functions (`size(list)`) or as methods (`list.size()`, `'text'.contains('t')`). They return CEL values and propagate errors when arguments are invalid.

### Collection helpers

- `size(value)` – Length of a string, bytes, or list.
- `sort(list)` – Returns a new list sorted using CEL ordering; non-comparable members yield `invalid_op` errors.
- `zip(list1, list2, ...)` – Zips multiple lists into a list of same-length tuples (shortest list wins); arguments must all be lists.
- `min(arg1, arg2, ...)` / `max(...)` – Vararg numeric/string comparator that returns the min/max; at least one argument required. Also callable as a method on a list (`list.min()`, `list.max()`).
- `sum(list)` – Returns the sum of all elements in a list; at least one element required. Also callable as a free function (`sum(a, b, ...)`).
- `flatten(list)` – Flattens one level of nested lists; non-list elements are kept as-is.
- `reverse(list)` – Returns a new list with elements in reverse order.
- `slice(list, start, end)` – Returns the sub-list `[start, end)`. Negative indices count from the end. `start` must be ≤ `end` or an argument error is raised.
- `unique(list)` – Returns a new list with duplicate elements removed, preserving first-occurrence order.

### String and text helpers

- `contains`, `containsI` – Substring containment (case-sensitive / case-insensitive).
- `startsWith`, `startsWithI`, `endsWith`, `endsWithI` – Prefix/suffix checks.
- `matches` – Returns `true` if a regex matches the entire string; invalid regex patterns raise a `value()` error.
- `matchCaptures` – Returns a list of capture groups (entire match first) or `null` if the regex does not match.
- `matchCapturesAll` – Returns a list of capture-group lists for every non-overlapping match of the regex, or an empty list if there are no matches.
- `matchReplace`, `matchReplaceOnce` – Regex replacement across all matches or only the first match.
- `remove` – Removes all non-overlapping occurrences of a literal substring.
- `replace` – Literal string replacement.
- `replaceI` – Case-insensitive literal string replacement (all occurrences).
- `indexOf(str, needle)` – Returns the character index of the first occurrence of `needle`, or `-1` if not found.
- `lastIndexOf(str, needle)` – Returns the character index of the last occurrence of `needle`, or `-1` if not found.
- `repeat(str, n)` – Returns the string repeated `n` times.
- `padStart(str, width[, char])` – Left-pads `str` to at least `width` characters using `char` (default `' '`). `char` must be a single character.
- `padEnd(str, width[, char])` – Right-pads `str` to at least `width` characters using `char` (default `' '`). `char` must be a single character.
- `split`, `rsplit` – Split on a literal delimiter from the left/right.
- `splitAt` – Splits at an index, returning `[left, right]`.
- `splitWhiteSpace` – Splits on any Unicode whitespace.
- `trim`, `trimStart`, `trimEnd` – Trim ASCII whitespace.
- `trimMatches(str, chars)` – Trims any characters in `chars` from both ends of the string.
- `trimStartMatches`, `trimEndMatches` – Trim a literal prefix/suffix repeatedly.
- `toLower`, `toUpper` – Case conversion.

All string helpers expect `this` to be a string; non-string inputs produce `value()` errors.

### Math & numeric helpers

- `abs(number)` – Absolute value for `int`, `uint`, and `double`.
- `sqrt(number)` – Square root returning `double`.
- `cbrt(number)` – Cube root returning `double`.
- `pow(base, exponent)` – Exponentiation for numeric combinations (integer exponents for integral bases).
- `exp(number)` – Returns e raised to the power of `number` as a `double`.
- `ln(number)` – Natural logarithm returning `double`.
- `log(number)` – Base-10 logarithm (`ilog10` for integers/unsigned integers).
- `lg(number)` – Base-2 logarithm.
- `ceil(number)`, `floor(number)`, `round(number)` – Standard rounding family; integral inputs are returned unchanged.
- `trunc(number)` – Truncates toward zero; `double` input returns `int`, integral inputs are returned unchanged.
- `clamp(value, min, max)` – Returns `min` if `value < min`, `max` if `value > max`, otherwise `value`.

### Time & date helpers

All time functions operate on `timestamp()` or `duration()` results. Where noted, a second optional argument is an IANA timezone string (e.g., `"America/Los_Angeles"`) which is resolved using `chrono_tz`.

- `getDate(timestamp[, timezone])` – Day of month (1–31).
- `getDayOfMonth(timestamp[, timezone])` – Zero-based day of month (0–30).
- `getDayOfWeek(timestamp[, timezone])` – Day of week (`0` = Sunday).
- `getDayOfYear(timestamp[, timezone])` – Zero-based day of year.
- `getFullYear(timestamp[, timezone])` – Four-digit year.
- `getMonth(timestamp[, timezone])` – Zero-based month (`0` = January).
- `getHours(timestamp | duration[, timezone])` – Hour of day or total hours of a duration.
- `getMinutes(timestamp | duration[, timezone])` – Minute of hour or total minutes of a duration.
- `getSeconds(timestamp | duration[, timezone])` – Second of minute or total seconds of a duration.
- `getMilliseconds(timestamp | duration[, timezone])` – Millisecond component or total milliseconds of a duration.
- `setFullYear(timestamp, year[, timezone])` – Returns a new UTC timestamp with the year set.
- `setMonth(timestamp, month[, timezone])` – Returns a new UTC timestamp with the zero-based month set.
- `setDate(timestamp, day[, timezone])` – Returns a new UTC timestamp with the day-of-month set.
- `setHours(timestamp, hour[, timezone])` – Returns a new UTC timestamp with the hour set.
- `setMinutes(timestamp, minute[, timezone])` – Returns a new UTC timestamp with the minute set.
- `setSeconds(timestamp, second[, timezone])` – Returns a new UTC timestamp with the second set.
- `setMilliseconds(timestamp, ms[, timezone])` – Returns a new UTC timestamp with the millisecond component set.
- `startOfDay(timestamp[, timezone])` – Returns a new UTC timestamp at midnight (00:00:00) of the same calendar day.
- `startOfMonth(timestamp[, timezone])` – Returns a new UTC timestamp at the first moment of the first day of the same month.
- `startOfYear(timestamp[, timezone])` – Returns a new UTC timestamp at the first moment of January 1st of the same year.
- `toRfc3339(timestamp[, timezone])` – RFC3339 string for the timestamp.
- `toTimestampString(timestamp[, timezone])` – Alias for `toRfc3339`.
- `now()` – Current UTC timestamp (no arguments).

### Unit conversion

- `uomConvert(value, from_unit, to_unit)` – Converts between supported units using the [`uom`](https://docs.rs/uom) crate. Units are case-insensitive and trimmed of leading `°`. Supported categories:
  - **Mass**: kilogram (`kg`), gram (`g`), milligram, pound (`lb`, `lbs`), ounce (`oz`), stone, slug, ton/tonne.
  - **Volume**: liter (`l`), milliliter, gallon, quart (liquid/dry), pint (liquid/dry), cup, fluid ounce, tablespoon, teaspoon, cubic meter/foot/yard.
  - **Speed**: meter per second (`m/s`), kilometer per hour (`km/h`, `kph`), mile per hour (`mph`), knot, foot per second (`ft/s`, `fps`).
  - **Temperature**: Celsius (`C`), Fahrenheit (`F`), Kelvin (`K`).

  Conversions only succeed within the same category. Invalid or mixed-unit requests raise argument errors.

### Miscellaneous helpers

- `size(value)` – See Collections.
- `sort(list)` – See Collections.
- `zip(list, ...)` – See Collections.

## Putting it together

```text
// Filter, map, and aggregate a bound list of structs.
accounts.map(a,
    a.balance_cents > 0,
    {
        'id': a.id,
        'balance': a.balance_cents / 100
    }
).reduce(total, acct, total + acct.balance, 0)
```

Combine macros and helpers freely. Errors or type mismatches surface as `CelError` instances, so guard with `has()` or `coalesce()` where appropriate.

## Extending the environment

You can bind additional values, functions, and macros via `BindContext::bind_param`, `bind_func`, and `bind_macro`. All defaults documented above remain available unless you intentionally replace them.
