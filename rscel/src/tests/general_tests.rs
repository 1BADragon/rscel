use crate::{
    compiler::{compiler::CelCompiler, string_tokenizer::StringTokenizer},
    BindContext, ByteCode, CelContext, CelError, CelValue, Program,
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde_json::Value;
use std::{assert, assert_eq, collections::HashMap, str::FromStr, sync::Arc};
use test_case::test_case;

#[test]
fn test_bad_func_call() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "foo(3)").unwrap();

    let res = ctx.exec("main", &exec_ctx);
    assert!(res.is_err());
}

#[test]
fn test_contains() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "\"hello there\".contains(\"hello\")")
        .unwrap();

    let _res = ctx.exec("main", &exec_ctx).unwrap();
}

#[test_case("3+3", 6; "add signed")]
#[test_case("4-3", 1; "sub signed")]
#[test_case("4u + 3u", 7u64; "add unsigned")]
#[test_case("7 % 2", 1; "test mod")]
#[test_case("(4+2) * (6-5)", 6; "test parens")]
#[test_case("4+2*6-5", 11; "test op order")]
#[test_case("4-2+5*2", 12; "test op order 2")]
#[test_case("[1, 2, 3].map(x, x+2)", vec![3, 4, 5]; "test map")]
#[test_case("[1, 2, 3].map(x, x % 2 == 1, x + 1)", vec![2, 4]; "test map 2")]
#[test_case("[1,2,3][1]", 2; "array index")]
#[test_case("[[1, 2], [3, 4]][1][0]", 3; "nested list chained index")]
#[test_case("{'a': {'b': 42}}['a']['b']", 42; "nested map chained index")]
#[test_case("{\"foo\": 3}.foo", 3; "obj dot access")]
#[test_case("size([1,2,3,4])", 4u64; "test list size")]
#[test_case("size('foo')", 3u64; "size string")]
#[test_case("size(b'foo')", 3u64; "size bytes")]
#[test_case("'foo'.size()", 3u64; "string size")]
#[test_case("b'foo'.size()", 3u64; "bytes size")]
#[test_case("true || false", true; "or")]
#[test_case("true || undefined", true; "or shortcut")]
#[test_case("false && undefined", false; "and shortcut")]
#[test_case("false && true", false; "and falsy")]
#[test_case("true && true", true; "and true")]
#[test_case("[1,2].map(x, x+1).map(x, x*2)", vec![4, 6]; "double map")]
#[test_case("\"hello world\".contains(\"hello\")", true; "test contains")]
#[test_case("\"hello WORLD\".containsI(\"hello\")", true; "test containsI")]
#[test_case("\"hello world\".endsWith(\"world\")", true; "test endsWith")]
#[test_case("'fooBaR'.endsWithI('bar')", true; "Test endsWithI")]
#[test_case("\"hello world\".startsWith(\"hello\")", true; "test startsWith")]
#[test_case("'FoObar'.startsWithI('foo')", true; "Test startsWithI")]
#[test_case("\"abc123\".matches(\"[a-z]{3}[0-9]{3}\")", true; "test matches method")]
#[test_case("string(1)", "1"; "test string")]
#[test_case("type(1)", CelValue::int_type(); "test type")]
#[test_case("4 > 5", false; "test gt")]
#[test_case("4 < 5", true; "test lt")]
#[test_case("4 >= 4", true; "test ge")]
#[test_case("5 <= 4", false; "test le")]
#[test_case("5 == 5", true; "test eq")]
#[test_case("5 != 5", false; "test ne")]
#[test_case("3 in [1,2,3,4,5]", true; "test in")]
#[test_case(r#"has({"foo": 3}.foo)"#, true; "test has")]
#[test_case("[1,2,3,4].all(x, x < 5)", true; "test all true")]
#[test_case("[1,2,3,4,5].all(x, x < 5)", false; "test all false")]
#[test_case("[1,2,3,4].exists(x, x < 3)", true; "test exists true")]
#[test_case("[1,2,3,4].exists(x, x == 5)", false; "test exists false")]
#[test_case("[1,2,3,4].exists_one(x, x == 4)", true; "test exists one true")]
#[test_case("[1,2,3,4].exists_one(x, x == 5)", false; "test exists one false")]
#[test_case("[1,2,3,4].filter(x, x % 2 == 0)", vec![2, 4]; "test filter")]
#[test_case("abs(-9)", 9; "abs1")]
#[test_case("abs(9u)", 9u32; "abs2")]
#[test_case("abs(-9.0)", 9f64; "abs3")]
#[test_case("sqrt(9.0)", 3.0; "sqrt float")]
#[test_case("sqrt(9)", 3.0; "sqrt int")]
#[test_case("sqrt(9u)", 3.0; "sqrt uint")]
#[test_case("pow(2, 2)", 4; "pow int int")]
#[test_case("pow(2u, 2u)", 4u64; "pow uint uint")]
#[test_case("pow(2u, 2.0)", 4u64; "pow uint float")]
#[test_case("pow(2u, 2)", 4u64; "pow uint int")]
#[test_case("pow(2, 2.0)", 4; "pow int float")]
#[test_case("pow(2, 2u)", 4; "pow int uint")]
#[test_case("pow(2.0, 2)", 4.0; "pow float int")]
#[test_case("pow(2.0, 2u)", 4.0; "pow float uint")]
#[test_case("pow(4.0, 0.5)", 2.0; "pow float float")]
#[test_case("log(1000.0)", 3.0; "log float")]
#[test_case("log(1)", 0; "log int")]
#[test_case("log(1u)", 0u64; "log unsigned")]
#[test_case("ceil(2)", 2; "ceil int")]
#[test_case("ceil(2u)", 2u64; "ceil uint")]
#[test_case("ceil(2.3)", 3; "ceil float")]
#[test_case("floor(2.7)", 2; "floor float")]
#[test_case("floor(3)", 3; "floor int")]
#[test_case("floor(3u)", 3u64; "floor uint")]
#[test_case("round(2.2)", 2; "round down")]
#[test_case("round(2.5)", 3; "round up")]
#[test_case("round(3)", 3; "round int")]
#[test_case("round(3u)", 3u64; "round uint")]
#[test_case("min(1,2,3)", 1; "min variadic")]
#[test_case("max(1,2,3)", 3; "max variadic")]
#[test_case("[3,1,2].min()", 1; "min list method")]
#[test_case("[3,1,2].max()", 3; "max list method")]
#[test_case("ln(1.0)", 0.0; "ln float")]
#[test_case("ln(1)", 0.0; "ln int")]
#[test_case("ln(1u)", 0.0; "ln uint")]
#[test_case("exp(0.0)", 1.0; "exp float")]
#[test_case("exp(0)", 1.0; "exp int")]
#[test_case("exp(0u)", 1.0; "exp uint")]
#[test_case("cbrt(27.0)", 3.0; "cbrt float")]
#[test_case("cbrt(8)", 2.0; "cbrt int")]
#[test_case("cbrt(8u)", 2.0; "cbrt uint")]
#[test_case("trunc(3.7)", 3; "trunc positive")]
#[test_case("trunc(-3.7)", -3; "trunc negative")]
#[test_case("trunc(5)", 5; "trunc int")]
#[test_case("trunc(5u)", 5u64; "trunc uint")]
#[test_case("clamp(5, 1, 10)", 5; "clamp in range")]
#[test_case("clamp(0, 1, 10)", 1; "clamp below min")]
#[test_case("clamp(11, 1, 10)", 10; "clamp above max")]
#[test_case("clamp(3.0, 1.0, 5.0)", 3.0; "clamp float in range")]
#[test_case("[1,2,3].reduce(curr, next, curr + next, 0)", 6; "reduce")]
#[test_case("{}", HashMap::new(); "empty object")]
#[test_case("[]", Vec::<CelValue>::new(); "empy list")]
#[test_case("has(foo) && foo > 10", false; "has works")]
#[test_case("true ? 4 : 3", 4; "ternary true")]
#[test_case("false ? 4 : 3", 3; "ternary false")]
#[test_case("2 * 4 * 8 * 72 / 144", 32; "long multiply operation")]
#[test_case("2 * 3 + 7", 13; "long mixed operation")]
#[test_case("true && false || true && true", true; "long logic operation")]
#[test_case("2 + 3 - 1", 4; "long add/sub operation")]
#[test_case("-2 + 4", 2; "neg pos addition")]
#[test_case("2 < 3 >= 1", true; "type prop: chained cmp")]
#[test_case("3 * 2 - 1 / 4 * 2", 6; "large op 2")]
#[test_case("true || unbound || unbound", true; "Or short cut")]
#[test_case("true == true || false == true && false", true; "Incorrect equality precedence")]
#[test_case("5 < 10 || 10 < 5 && false", true; "Incorrect less-than precedence")]
#[test_case("true || false && false", true; "Incorrect AND precedence")]
#[test_case("false && true || true", true; "Incorrect OR precedence")]
#[test_case("5 + 5 == 10 || 10 - 5 == 5 && false", true; "Incorrect addition precedence")]
#[test_case("6 / 2 == 3 || 2 * 3 == 6 && false", true; "Incorrect division precedence")]
#[test_case("(true || false) && false", false; "Incorrect parentheses precedence")]
#[test_case("'foo' in 'foot'", true; "in string operator")]
#[test_case("'foot' in 'foo'", false; "in string operator false")]
#[test_case("type(3) == type(3)", true; "type eq")]
#[test_case("type(null) == null_type", true; "null_type eq")]
#[test_case("type(3) == int", true; "int type eq")]
#[test_case("type(3u) == uint", true; "uint type eq")]
#[test_case("type('foo') == string", true; "string type eq")]
#[test_case("type(true) == bool", true; "bool type eq true")]
#[test_case("type(false) == bool", true; "bool type eq false")]
#[test_case("type(3.2) == double", true; "double type eq")]
#[test_case("type(3.2) == float", true; "float type eq")]
#[test_case("type(true) == double", false; "bool type neq")]
#[test_case("type(true) != double", true; "bool type neq 2")]
#[test_case("type([1,2,3]) == type([])", true; "list type neq")]
#[test_case("type({'foo': 3}) == type({})", true; "map type neq")]
#[test_case("coalesce()", CelValue::from_null(); "coalesce none")]
#[test_case("coalesce(null, 3)", 3; "coalesce explicit null")]
#[test_case("coalesce(foo, 4)", 4; "coalesce unbound var")]
#[test_case("coalesce(1, 2, 3)", 1; "coalesce first val ok")]
#[test_case(".1", 0.1; "dot leading floating point")]
#[test_case("-.1", -0.1; "neg dot leading floating point")]
#[test_case("2+3 in [5]", true; "check in binding")]
#[test_case("foo.b || true", true; "Error bypassing")]
#[test_case(r#""\u00fc""#, "ü"; "Test unicode short lower")]
#[test_case(r#""\u00FC""#, "ü"; "Test unicode short upper")]
#[test_case(r#""\U000000fc""#, "ü"; "Test unicode long lower")]
#[test_case(r#""\U000000FC""#, "ü"; "Test unicode long upper")]
#[test_case(r#""\x48""#, "H"; "Test hex escape lower")]
#[test_case(r#""\X48""#, "H"; "Test hex escape upper")]
#[test_case("'   foo   '.trim()", "foo"; "Test trim")]
#[test_case("'   foo   '.trimStart()", "foo   "; "Test trimStart")]
#[test_case("'   foo   '.trimEnd()", "   foo"; "Test trimEnd")]
#[test_case("'foo'.toUpper()", "FOO"; "test toUpper")]
#[test_case("'FOO'.toLower()", "foo"; "test toLower")]
#[test_case(r#"'foo   bar\t\tbaz'.splitWhiteSpace()"#, vec!["foo", "bar", "baz"]; "test splitWhiteSpace")]
#[test_case("{'foo': x}.map(k, k)", vec!["foo"]; "test map on map")]
#[test_case("{'foo': x, 'bar': y}.filter(k, k == 'foo')", vec!["foo"]; "test filter on map")]
#[test_case(r#"f"{3}""#, "3"; "test basic format string")]
#[test_case(r#"f"{({"foo": 3}).foo)}""#, "3"; "test fstring with map")]
#[test_case(r#"f"{[1,2,3][2]}""#, "3"; "test fstring with list")]
#[test_case("timestamp('2024-07-30 12:00:00+00:00') - timestamp('2024-07-30 11:55:00+00:00') == duration('5m')", true; "test timestamp sub 1")]
#[test_case("timestamp('2024-07-30 11:55:00+00:00') - timestamp('2024-07-30 12:00:00+00:00')", Duration::new(-300, 0).unwrap(); "test timestamp sub 2")]
#[test_case("timestamp('2023-12-25T12:00:00Z').getDayOfMonth()", 24; "getDayOfMonth")]
#[test_case("timestamp('2023-12-25T7:00:00Z').getDayOfMonth('America/Los_Angeles')", 23; "getDayOfMonth with timezone")]
#[test_case("int(1)", 1; "identity -- int")]
#[test_case("uint(1u)", 1u64; "identity -- uint")]
#[test_case("double(5.5)", 5.5; "identity -- double")]
#[test_case("string('hello')", "hello"; "identity -- string")]
#[test_case("bytes(bytes('abc'))", crate::types::CelBytes::from_vec(vec![97u8, 98u8, 99u8]); "identity -- bytes 1")]
#[test_case("bytes(b'abc')", crate::types::CelBytes::from_vec(vec![97u8, 98u8, 99u8]); "identity -- bytes 2")]
#[test_case("duration(duration('100s')) == duration('100s')", true; "identity -- duration")]
#[test_case("duration('2h') + duration('1h1m') >= duration('3h')", true; "duration add + comp")]
#[test_case("timestamp(timestamp(100000000)) == timestamp(100000000)", true; "identity -- timestamp")]
#[test_case("bool(true)", true; "bool true")]
#[test_case("bool(false)", false; "bool false")]
#[test_case("bool('1')", true; "'1' -> bool")]
#[test_case("bool('t')", true; "'t' -> bool")]
#[test_case("bool('true')", true; "'true' -> bool 1")]
#[test_case("bool('TRUE')", true; "'TRUE' -> bool 2")]
#[test_case("bool('True')", true; "'True' -> bool 3")]
#[test_case("bool('0')", false; "'0' -> bool")]
#[test_case("bool('f')", false; "'f' -> bool")]
#[test_case("bool('false')", false; "'false' -> bool 1")]
#[test_case("bool('FALSE')", false; "'FALSE' -> bool 2")]
#[test_case("bool('False')", false; "'False' -> bool 3")]
#[test_case("!true", false; "not true")]
#[test_case("!false", true; "not false")]
#[test_case("1 + 2 == 3 && 4 + 5 == 9", true; "and operation with expressions")]
#[test_case("1 + 2 == 3 || 4 + 5 == 10", true; "or operation with expressions")]
#[test_case("! (1 + 2 == 4)", true; "negated expression")]
#[test_case("size([1, 2, 3].filter(x, x > 1)) == 2", true; "filter and size")]
#[test_case("max(1, 2, 3) + min(4, 5, 6) == 7", true; "max and min")]
#[test_case("['hello', 'world'].reduce(curr, next, curr + ' ' + next, '')", " hello world"; "reduce with strings")]
#[test_case("timestamp('2024-07-30 12:00:00Z') > timestamp('2023-07-30 12:00:00Z')", true; "timestamp comparison")]
#[test_case("{'a': 1, 'b': 2, 'c': 3}.filter(k, k == 'b').size() == 1", true; "filter on map with modulo")]
#[test_case("[1, 2, 3, 4].all(x, x < 5) && [1, 2, 3, 4].exists(x, x == 3)", true; "all and exists")]
#[test_case("coalesce(null, null, 'hello', null) == 'hello'", true; "coalesce with multiple nulls")]
#[test_case("duration('3h').getHours()", 3; "duration.getHours")]
#[test_case("duration('1s234ms').getMilliseconds()", 234; "duration.getMilliseconds")]
#[test_case("duration('1h30m').getMinutes()", 90; "duration.getMinutes")]
#[test_case("duration('1m30s').getSeconds()", 90; "duration.getSeconds")]
#[test_case("match 'foo' {case int: false, case _: true}", true; "match else")]
#[test_case("match 3 { case int: true, case _: false}", true; "match int" )]
#[test_case("match 2.0 { case float: true, case _: flase}", true; "match float")]
#[test_case("match 'foo' { case string: true, case _: false}", true; "match string")]
#[test_case("match false { case bool: true, case _: false}", true; "match bool")]
#[test_case("match 3 { case 3: true, case _: flase}", true; "match int literal")]
#[test_case("match 3.0 { case 3.0: true, case _: flase}", true; "match float literal")]
#[test_case("match '3' { case '3': true, case _: flase}", true; "match string literal")]
#[test_case("match 3 { case >2: true, case _: false}", true; "match greater than")]
#[test_case("match 3 { case >=2: true, case _: false}", true; "match greater equal")]
#[test_case("match 3 { case >=3: true, case _: false}", true; "match greater equal equal")]
#[test_case("match 3 { case <2: false, case _: true}", true; "match less than")]
#[test_case("match 3 { case <=2: false, case _: true}", true; "match less equal")]
#[test_case("match 3 { case <=3: true, case _: false}", true; "match less equal equal")]
#[test_case("[3,4,2,1].sort()", vec![1,2,3,4]; "sort int")]
#[test_case("[3.4, 2.1, 4.8].sort()", vec![2.1, 3.4, 4.8]; "sort float")]
#[test_case("['apple', 'cookie', 'bananas'].sort()", vec!["apple", "bananas", "cookie"]; "sort string")]
#[test_case("'123LF3040'.remove('LF')", "1233040"; "string remove")]
#[test_case("'123M5'.replace('M', '4')", "12345"; "string replace")]
#[test_case("'12131415'.rsplit('1')", vec!["5", "4", "3", "2", ""]; "string rsplit")]
#[test_case("'12131415'.split('1')", vec!["", "2", "3", "4", "5"]; "string split")]
#[test_case("'123456'.splitAt(3)", vec!["123", "456"]; "string splitAt signed index")]
#[test_case("'123456'.splitAt(3u)", vec!["123", "456"]; "string splitAt unsigned index")]
#[test_case("'12345LF'.trimEndMatches('LF')", "12345"; "string trimEndMatches")]
#[test_case("'LF12345'.trimStartMatches('LF')", "12345"; "string trimStartMatches")]
#[test_case("zip([1, 2, 3], ['a', 'b', 'c'])",
    CelValue::from_val_slice(&[
        CelValue::from_val_slice(&[1.into(), "a".into()]),
        CelValue::from_val_slice(&[2.into(), "b".into()]),
        CelValue::from_val_slice(&[3.into(), "c".into()])]
    ); "zip")]
#[test_case(r#"'123abc555'.matchCaptures('([0-9]+)([a-z]+)555')"#, vec!["123abc555", "123", "abc"]; "string match captures")]
#[test_case("'abab'.matchReplaceOnce('(?<first>a)(?<last>b)', '${last}${first}')", "baab"; "string matchReplaceOnce")]
#[test_case("'abab'.matchReplace('(?<first>a)(?<last>b)', '${last}${first}')", "baba"; "string matchReplace")]
#[test_case("'hello world'.indexOf('world')", 6i64; "string indexOf found")]
#[test_case("'hello world'.indexOf('xyz')", -1i64; "string indexOf not found")]
#[test_case("'abcabc'.lastIndexOf('b')", 4i64; "string lastIndexOf found")]
#[test_case("'abcabc'.lastIndexOf('xyz')", -1i64; "string lastIndexOf not found")]
#[test_case("'ha'.repeat(3)", "hahaha"; "string repeat int")]
#[test_case("'ab'.repeat(0)", ""; "string repeat zero")]
#[test_case("'Hello World'.replaceI('hello', 'Hi')", "Hi World"; "string replaceI case insensitive")]
#[test_case("'HELLO hello'.replaceI('hello', 'bye')", "bye bye"; "string replaceI replaces all")]
#[test_case("'--hello--'.trimMatches('-')", "hello"; "string trimMatches")]
#[test_case("'hello'.padStart(8)", "   hello"; "string padStart space")]
#[test_case("'hello'.padStart(8, '0')", "000hello"; "string padStart custom char")]
#[test_case("'hello'.padStart(3)", "hello"; "string padStart noop when already long enough")]
#[test_case("'hello'.padEnd(8)", "hello   "; "string padEnd space")]
#[test_case("'hello'.padEnd(8, '!')", "hello!!!"; "string padEnd custom char")]
#[test_case("'hello'.padEnd(3)", "hello"; "string padEnd noop when already long enough")]
// list functions
#[test_case("[3, 1, 2].reverse()", vec![2, 1, 3]; "list reverse")]
#[test_case("[[1, 2], [3, 4]].flatten()", vec![1, 2, 3, 4]; "list flatten one level")]
#[test_case("[1, [2, [3]]].flatten()", CelValue::from_val_slice(&[1.into(), 2.into(), CelValue::from_val_slice(&[3.into()])]); "list flatten one level only")]
#[test_case("[1, 2, 1, 3, 2].unique()", vec![1, 2, 3]; "list unique")]
#[test_case("[1, 2, 3, 4, 5].slice(1, 3)", vec![2, 3]; "list slice")]
#[test_case("[1, 2, 3, 4, 5].slice(0, -1)", vec![1, 2, 3, 4]; "list slice negative end")]
#[test_case("[1, 2, 3].sum()", 6i64; "list sum ints")]
#[test_case("[1.0, 2.5, 3.5].sum()", 7.0f64; "list sum floats")]
// macros
#[test_case("[1, 2, 3, 4, 5].find(x, x > 3)", 4i64; "macro find first match")]
#[test_case("[1, 2, 3].find(x, x > 10)", CelValue::Null; "macro find no match returns null")]
#[test_case("[1, 2, 3, 4, 5].count(x, x % 2 == 0)", 2i64; "macro count even")]
#[test_case("[1, 2, 3].count(x, x > 0)", 3i64; "macro count all match")]
#[test_case("[[1, 2], [3, 4]].flatMap(x, x)", vec![1, 2, 3, 4]; "macro flatMap identity")]
#[test_case("[1, 2, 3].flatMap(x, [x, x * 10])", vec![1, 10, 2, 20, 3, 30]; "macro flatMap expand")]
// timestamps
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfDay().toRfc3339()", "2024-07-15T00:00:00+00:00"; "startOfDay UTC")]
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfDay('America/New_York').toRfc3339()", "2024-07-15T04:00:00+00:00"; "startOfDay with timezone")]
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfMonth().toRfc3339()", "2024-07-01T00:00:00+00:00"; "startOfMonth UTC")]
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfMonth('America/New_York').toRfc3339()", "2024-07-01T04:00:00+00:00"; "startOfMonth with timezone")]
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfYear().toRfc3339()", "2024-01-01T00:00:00+00:00"; "startOfYear UTC")]
#[test_case("timestamp('2024-07-15T14:32:07Z').startOfYear('America/New_York').toRfc3339()", "2024-01-01T05:00:00+00:00"; "startOfYear with timezone")]
#[test_case("timestamp('2023-01-01T04:00:00-01:00').toRfc3339()", "2023-01-01T05:00:00+00:00"; "timestamp to_rfc3339")]
#[test_case("timestamp('2023-01-01T04:00:00-01:00').toRfc3339('EST')", "2023-01-01T00:00:00-05:00"; "timestamp to_rfc3339 with timezone EST")]
#[test_case("timestamp('2023-01-01T04:00:00-01:00').toRfc3339('America/New_York')", "2023-01-01T00:00:00-05:00"; "timestamp to_rfc3339 with timezone America/New_York")]
#[test_case(r#"timestamp("2026-02-19T13:00:04-00:00").format("%B %d, %Y %H:%M")"#, "February 19, 2026 13:00"; "format timestamp")]
#[test_case(r#"timestamp("2026-02-19T13:00:04-00:00").format("America/Los_Angeles", "%B %d, %Y %H:%M")"#, "February 19, 2026 05:00"; "format timestamp w/ timezone")]
#[test_case("type([]) == list", true; "list type")]
#[test_case("type({}) == obj", true; "obj type")]
#[test_case("format(42, 'd')", "42"; "format int default")]
#[test_case("format(42, '08d')", "00000042"; "format int zero padded")]
#[test_case("format(42, 'x')", "2a"; "format int hex lower arg form")]
#[test_case("format(42, 'X')", "2A"; "format int hex upper")]
#[test_case("format(42, 'o')", "52"; "format int octal")]
#[test_case("format(42, 'b')", "101010"; "format int binary")]
#[test_case("format(255, '08b')", "11111111"; "format int binary zero padded")]
#[test_case("format(3.14159, '.2f')", "3.14"; "format float fixed 2dp")]
#[test_case("format(3.14159, '.4f')", "3.1416"; "format float fixed 4dp")]
#[test_case("format(0.000042, '.2e')", "4.20e-5"; "format float scientific lower arg form")]
#[test_case("format(0.000042, '.2E')", "4.20E-5"; "format float scientific upper")]
#[test_case("format(1234567.0, '.2f')", "1234567.00"; "format float large fixed")]
#[test_case("format(1.0, '10.3f')", "     1.000"; "format float space padded")]
#[test_case("format(1.0, '010.3f')", "000001.000"; "format float zero padded")]
#[test_case("true.format('')", "true"; "format bool true")]
#[test_case("false.format('')", "false"; "format bool false")]
#[test_case("format(true, 'ignored')", "true"; "format bool spec ignored arg form")]
#[test_case("'hello world'.format('.5')", "hello"; "format string truncate")]
#[test_case("'hello world'.format('.5e')", "hello…"; "format string truncate ellipsis")]
#[test_case("format('hello', '.10')", "hello"; "format string truncate noop shorter than width")]
#[test_case("'hello world'.format('')", "hello world"; "format string no spec")]
#[test_case("'café'.format('.3')", "caf"; "format string truncate unicode")]
#[test_case("'café'.format('.3e')", "caf…"; "format string truncate unicode ellipsis")]
#[test_case("b'deadbeef'.format('')", "6465616462656566"; "format bytes default this form")]
#[test_case("format(b'\\x00\\xff', '')", "00ff"; "format bytes hex arg form")]
#[test_case("format(null, '')", "null"; "format null arg form")]
#[test_case("format(null)", "null"; "format null no spec")]
#[test_case("duration('125s').format('seconds')", "125s"; "format duration seconds unit")]
#[test_case("duration('125s').format('minutes')", "2m"; "format duration minutes floor")]
#[test_case("duration('3600s').format('hours')", "1h"; "format duration hours")]
#[test_case("duration('86400s').format('days')", "1d"; "format duration days")]
#[test_case("duration('90061s').format('{d}d {h}h {m}m {s}s')", "1d 1h 1m 1s"; "format duration template all units")]
#[test_case("duration('3754s').format('{h}h {m}m {s}s')", "1h 2m 34s"; "format duration template hms")]
#[test_case("duration('65s').format('{m}m {s}s')", "1m 5s"; "format duration template ms")]
#[test_case("duration('30s').format('{h}h {m}m {s}s')", "0h 0m 30s"; "format duration template zeros shown")]
#[test_case("duration('125s').format('')", "2m"; "format duration default largest unit")]
#[test_case("format(duration('30s'), 's')", "30s"; "format duration seconds arg form")]
#[test_case("[1, 2, 3].format('')", "[1, 2, 3]"; "format list ints")]
#[test_case("format([true, 'hello', 1])", "[true, hello, 1]"; "format list mixed")]
#[test_case("format([[1, 2], [3, 4]])", "[[1, 2], [3, 4]]"; "format list nested")]
#[test_case("{'a': 1}.format('')", "{a: 1}"; "format map this form")]
#[test_case("format({'x': true})", "{x: true}"; "format map arg form")]
#[test_case("format(42, '10d')", "        42"; "format int space padded right aligned")]
#[test_case("format(42, '010d')", "0000000042"; "format int zero padded width 10")]
#[test_case("format(3.14, '10.2f')", "      3.14"; "format float space padded width 10 prec 2")]
#[test_case("format(3.14, '010.2f')", "0000003.14"; "format float zero padded width 10 prec 2")]
#[test_case("format(-3.14, '010.2f')", "-000003.14"; "format float zero padded negative width 10 prec 2")]
#[test_case("format(1.0, '010.3f')", "000001.000"; "format float zero padded width 10 prec 3")]
#[test_case("format(255, '010x')", "00000000ff"; "format int hex zero padded width 10")]
#[test_case("format(255, '10x')", "        ff"; "format int hex space padded width 10")]
#[test_case("format(7, '08b')", "00000111"; "format int binary zero padded width 8")]
#[test_case("format(7, '8b')", "     111"; "format int binary space padded width 8")]
fn test_equation(prog: &str, res: impl Into<CelValue>) {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", prog).unwrap();

    let eval_res = ctx.exec("main", &exec_ctx).unwrap();
    assert_eq!(eval_res, res.into());
}

#[test]
fn test_timestamp() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", r#"timestamp("2023-04-20T12:00:00Z")"#)
        .unwrap();
    let eval_res = ctx.exec("main", &exec_ctx).unwrap();

    let dt = DateTime::parse_from_rfc3339("2023-04-20T12:00:00Z").unwrap();
    assert_eq!(eval_res, dt.into());
}

#[test]
fn test_timestamp_now() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "timestamp()").unwrap();
    let eval_res = ctx.exec("main", &exec_ctx).unwrap();

    let CelValue::TimeStamp(ts) = eval_res else {
        panic!("timestamp() without arguments should return a timestamp");
    };

    let now = Utc::now();
    let delta = now.signed_duration_since(ts).num_seconds().abs();
    assert!(
        delta <= 5,
        "timestamp() should return current time. Delta {} seconds",
        delta
    );
}

#[test]
fn test_now_function() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "now()").unwrap();
    let eval_res = ctx.exec("main", &exec_ctx).unwrap();

    let CelValue::TimeStamp(ts) = eval_res else {
        panic!("now() should return a timestamp");
    };

    let now = Utc::now();
    let delta = now.signed_duration_since(ts).num_seconds().abs();
    assert!(delta <= 5, "now() delta too large: {delta}s");
}

#[test]
fn test_timeduration() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str(
        "main",
        r#"timestamp("2023-04-20T12:00:00Z") + duration("1h")"#,
    )
    .unwrap();
    let eval_res = ctx.exec("main", &exec_ctx).unwrap();

    let dt = DateTime::parse_from_rfc3339("2023-04-20T13:00:00Z").unwrap();
    assert_eq!(eval_res, dt.into());
}

#[test]
fn test_binding() {
    let mut ctx = CelContext::new();
    let mut binding = BindContext::new();

    ctx.add_program_str("main", "foo + 9").unwrap();

    binding.bind_param("foo", 3.into());
    assert_eq!(ctx.exec("main", &binding).unwrap(), 12.into());
}

#[test]
fn test_dict_binding() {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    ctx.add_program_str("func1", "foo.bar + 4").unwrap();
    ctx.add_program_str("func2", "foo.bar % 4").unwrap();
    ctx.add_program_str("func3", "foo.bar").unwrap();

    let mut foo: HashMap<String, CelValue> = HashMap::new();
    foo.insert("bar".to_owned(), 7.into());
    exec_ctx.bind_param("foo", foo.into());

    assert_eq!(ctx.exec("func1", &exec_ctx).unwrap(), 11.into());
    assert_eq!(ctx.exec("func2", &exec_ctx).unwrap(), 3.into());
    assert_eq!(ctx.exec("func3", &exec_ctx).unwrap(), 7.into());
}

#[test]
fn test_serialization() {
    let json_str = {
        let mut tokenizer = StringTokenizer::with_input("4+7*2");
        let prog = CelCompiler::with_tokenizer(&mut tokenizer)
            .compile()
            .unwrap();
        serde_json::to_string(&prog).unwrap()
    };

    let prog: Program = serde_json::from_str(&json_str).unwrap();

    let mut cel = CelContext::new();
    cel.add_program("main", prog);
    let bindings = BindContext::new();

    assert_eq!(cel.exec("main", &bindings).unwrap(), 18.into());
}

#[test]
fn test_nested() {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    ctx.add_program_str("foo", "val + 3").unwrap();
    ctx.add_program_str("bar", "foo * 3").unwrap();

    exec_ctx.bind_param("val", 7.into());

    assert_eq!(ctx.exec("bar", &exec_ctx).unwrap(), 30.into());
}

#[test]
fn test_call_depth_failure() {
    let mut ctx = CelContext::new();
    let exec = BindContext::new();

    ctx.add_program_str("entry", "entry + 3").unwrap();

    assert!(ctx.exec("entry", &exec).is_err());
}

#[test]
fn test_has_through() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str("entry", "has(foo) ? foo + 3 : 42")
        .unwrap();

    assert_eq!(ctx.exec("entry", &exec).unwrap(), 42.into());

    exec.bind_param("foo", 10.into());
    assert_eq!(ctx.exec("entry", &exec).unwrap(), 13.into());

    ctx.add_program_str("entry2", "has(a.b.c)").unwrap();
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), false.into());

    let mut a = HashMap::<String, CelValue>::new();
    exec.bind_param("a", a.clone().into());
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), false.into());

    let mut b = HashMap::<String, CelValue>::new();
    b.insert("c".to_string(), 4.into());
    a.insert("b".to_string(), b.into());
    exec.bind_param("a", a.into());
    assert_eq!(ctx.exec("entry2", &exec).unwrap(), true.into());
}

#[test]
fn test_object_access_in_array() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str("entry", "my_list[0].foo").unwrap();

    let mut obj_map = HashMap::<String, CelValue>::new();
    obj_map.insert("foo".to_owned(), "value".into());

    let obj = CelValue::from_val_slice(&vec![obj_map.into()]);
    exec.bind_param("my_list", obj);

    assert_eq!(ctx.exec("entry", &exec).unwrap(), "value".into());
}

#[test]
fn test_chained_index_on_binding() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    // data = {"users": [{"name": "alice", "scores": [10, 20, 30]},
    //                   {"name": "bob",   "scores": [40, 50, 60]}]}
    let data: CelValue = serde_json::from_str::<Value>(
        r#"{"users": [{"name": "alice", "scores": [10, 20, 30]},
                      {"name": "bob",   "scores": [40, 50, 60]}]}"#,
    )
    .unwrap()
    .into();
    exec.bind_param("data", data);

    ctx.add_program_str("user_name", "data['users'][1]['name']")
        .unwrap();
    ctx.add_program_str("score", "data['users'][0]['scores'][2]")
        .unwrap();

    assert_eq!(ctx.exec("user_name", &exec).unwrap(), "bob".into());
    assert_eq!(ctx.exec("score", &exec).unwrap(), 30.into());
}

#[test]
fn test_has_in_reduce() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str(
        "entry",
        "my_list.reduce(curr, next, curr + int(has(next.foo)), 0)",
    )
    .unwrap();

    let obj: CelValue = serde_json::from_str::<Value>("[{\"foo\": 1}, {}, {\"foo\": 1}]")
        .unwrap()
        .into();

    exec.bind_param("my_list", obj.into());

    assert_eq!(ctx.exec("entry", &exec).unwrap(), 2.into());
}

#[test]
#[allow(deprecated)]
fn test_timestamp_functions() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    let dt = Utc
        .ymd(2024, 01, 10)
        .and_hms_milli_opt(8, 57, 45, 123)
        .unwrap();
    exec.bind_param("time", CelValue::from_timestamp(dt));

    let progs = [
        ("time.getDate()", 10),
        ("time.getDate('HST')", 9),
        ("time.getDayOfMonth()", 9),
        ("time.getDayOfMonth('US/Pacific')", 9),
        ("time.getDayOfWeek()", 3),
        ("time.getDayOfWeek('US/Pacific')", 4),
        ("time.getDayOfYear()", 9),
        ("time.getDayOfYear('US/Pacific')", 9),
        ("time.getFullYear()", 2024),
        ("time.getFullYear('US/Pacific')", 2024),
        ("time.getHours()", 8),
        ("time.getHours('US/Pacific')", 0),
        ("time.getMilliseconds()", 123),
        ("time.getMilliseconds('US/Pacific')", 123),
        ("time.getMinutes()", 57),
        ("time.getMinutes('US/Pacific')", 57),
        ("time.getMonth()", 0),
        ("time.getMonth('US/Pacific')", 0),
        ("time.getSeconds()", 45),
        ("time.getSeconds('US/Pacific')", 45),
    ];

    for prog in progs.iter() {
        ctx.add_program_str("entry", prog.0).unwrap();

        let res = ctx.exec("entry", &exec).unwrap();
        println!("{}:{} == {}", prog.0, res, prog.1);
        assert!(res == prog.1.into());
    }
}

#[test]
#[allow(deprecated)]
fn test_timestamp_set_functions() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    let dt = Utc
        .ymd(2024, 01, 10)
        .and_hms_milli_opt(8, 57, 45, 123)
        .unwrap();
    exec.bind_param("time", CelValue::from_timestamp(dt));

    let progs = [
        (
            "time.setFullYear(2025).toRfc3339()",
            "2025-01-10T08:57:45.123+00:00",
        ),
        (
            "time.setFullYear(2025, 'US/Pacific').toRfc3339()",
            "2025-01-10T08:57:45.123+00:00",
        ),
        (
            "time.setMonth(5).toRfc3339()",
            "2024-06-10T08:57:45.123+00:00",
        ),
        (
            "time.setMonth(5, 'US/Pacific').toRfc3339()",
            "2024-06-10T07:57:45.123+00:00",
        ),
        (
            "time.setDate(11).toRfc3339()",
            "2024-01-11T08:57:45.123+00:00",
        ),
        (
            "time.setDate(11, 'US/Pacific').toRfc3339()",
            "2024-01-11T08:57:45.123+00:00",
        ),
        (
            "time.setHours(2).toRfc3339()",
            "2024-01-10T02:57:45.123+00:00",
        ),
        (
            "time.setHours(5, 'US/Pacific').toRfc3339()",
            "2024-01-10T13:57:45.123+00:00",
        ),
        (
            "time.setMinutes(30).toRfc3339()",
            "2024-01-10T08:30:45.123+00:00",
        ),
        (
            "time.setMinutes(30, 'US/Pacific').toRfc3339()",
            "2024-01-10T08:30:45.123+00:00",
        ),
        (
            "time.setSeconds(9).toRfc3339()",
            "2024-01-10T08:57:09.123+00:00",
        ),
        (
            "time.setSeconds(9, 'US/Pacific').toRfc3339()",
            "2024-01-10T08:57:09.123+00:00",
        ),
        (
            "time.setMilliseconds(987).toRfc3339()",
            "2024-01-10T08:57:45.987+00:00",
        ),
        (
            "time.setMilliseconds(987, 'US/Pacific').toRfc3339()",
            "2024-01-10T08:57:45.987+00:00",
        ),
    ];

    for (prog, expected) in progs.iter() {
        ctx.add_program_str("entry", prog).unwrap();

        let res = ctx.exec("entry", &exec).unwrap();
        println!("{}:{} == {}", prog, res, expected);
        assert!(res == (*expected).into());
    }
}

#[test]
fn test_coalesce() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    exec.bind_params_from_json_obj(Value::from_str("{\"foo\": 4, \"bar\":{\"a\": 3}}").unwrap())
        .unwrap();

    ctx.add_program_str("prog1", "coalesce(foo, 3)").unwrap();
    ctx.add_program_str("prog2", "coalesce(bar.a, 4)").unwrap();
    ctx.add_program_str("prog3", "coalesce(bar.b, bar.a)")
        .unwrap();

    assert_eq!(ctx.exec("prog1", &exec).unwrap(), 4.into());
    assert_eq!(ctx.exec("prog2", &exec).unwrap(), 3.into());
    assert_eq!(ctx.exec("prog3", &exec).unwrap(), 3.into());
}

#[test]
fn test_usage_running_example() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    ctx.add_program_str("main", "greeting + ' ' + subject")
        .unwrap();
    exec.bind_param("greeting", "hello".into());
    exec.bind_param("subject", "world".into());

    let value = ctx.exec("main", &exec).unwrap();
    assert_eq!(value, "hello world".into());
}

#[test]
fn test_usage_putting_it_together_example() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();

    let accounts = serde_json::from_str::<Value>(
        r#"[{"id":"a1","balance_cents":150},{"id":"b2","balance_cents":0},{"id":"c3","balance_cents":250}]"#,
    )
    .unwrap();
    exec.bind_param("accounts", accounts.into());

    ctx.add_program_str(
        "main",
        "accounts.map(a, a.balance_cents > 0, {'id': a.id, 'balance': a.balance_cents / 100}).reduce(total, acct, total + acct.balance, 0)",
    )
    .unwrap();

    let value = ctx.exec("main", &exec).unwrap();
    assert_eq!(value, 3.into());
}

#[test]
fn test_dyn_value() {
    let mut ctx = CelContext::new();
    let mut exec = BindContext::new();
    let mut exec2 = BindContext::new();

    ctx.add_program_str("main", "foo.bar")
        .expect("Failed to compile prog");
    ctx.add_program_str("prog2", "foo[\"bar\"]")
        .expect("Failed to compile prog2");
    ctx.add_program_str("prog3", "e == e")
        .expect("Failed to compile prog 3");

    let mut inner_map = HashMap::new();
    inner_map.insert("bar".to_string(), 5.into());
    let foo = CelValue::from_dyn(Arc::new(CelValue::from_map(inner_map)));
    exec.bind_param("foo", foo);

    exec.bind_param("e", CelValue::from_dyn(Arc::new(CelValue::from_int(4))));

    let mut inner_map = HashMap::new();
    inner_map.insert("bar".to_string(), 5.into());
    let foo = CelValue::from_map(inner_map);

    exec2.bind_param("foo", foo);
    assert_eq!(ctx.exec("main", &exec).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog2", &exec).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog2", &exec2).unwrap(), 5.into());
    assert_eq!(ctx.exec("prog3", &exec).unwrap(), true.into());
}

#[test]
fn test_keywords_as_access_idents() {
    let mut ctx = CelContext::new();

    ctx.add_program_str("main", "foo.timestamp")
        .expect("Failed to compile program");

    let mut exec1 = BindContext::new();
    let mut map1 = HashMap::new();
    map1.insert("timestamp".to_string(), 4.into());
    exec1.bind_param("foo", map1.into());

    assert_eq!(ctx.exec("main", &exec1).unwrap(), 4.into());

    let mut exec2 = BindContext::new();
    let map2 = HashMap::new();
    exec2.bind_param("foo", map2.into());

    match ctx.exec("main", &exec2) {
        Err(CelError::Attribute { .. }) => {}
        _ => panic!(),
    }
}

#[test]
fn test_map_literal_member_access_const_folds() {
    // {'a': 1}.a should be folded at compile time to a single Push(Int(1)),
    // with no Access instruction emitted.
    let prog = Program::from_source("{'a': 1}.a").unwrap();
    let bc = prog.bytecode();

    assert!(
        !bc.iter().any(|op| matches!(op, ByteCode::Access)),
        "expected no Access instruction, got:\n{}",
        prog.dumps_bc()
    );
    assert_eq!(
        bc.as_slice(),
        &[ByteCode::Push(CelValue::Int(1))],
        "expected single Push(Int(1)), got:\n{}",
        prog.dumps_bc()
    );
}

#[test]
fn test_match_captures_all() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", r#"'one1two2three3'.matchCapturesAll('([a-z]+)([0-9])')"#)
        .unwrap();

    let result = ctx.exec("main", &exec_ctx).unwrap();

    let expected = CelValue::from_val_slice(&[
        CelValue::from_val_slice(&["one1".into(), "one".into(), "1".into()]),
        CelValue::from_val_slice(&["two2".into(), "two".into(), "2".into()]),
        CelValue::from_val_slice(&["three3".into(), "three".into(), "3".into()]),
    ]);

    assert_eq!(result, expected);
}
