mod ast;
mod error;
mod grammar;
mod traits;

pub use traits::ToSql;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ToSqlError;
    use rscel::Program;
    use test_case::test_case;

    fn cel_to_sql(cel_expr: &str) -> Result<String, ToSqlError> {
        let program = Program::from_source(cel_expr)
            .map_err(|_| ToSqlError::unsupported("Failed to parse CEL expression"))?;
        let builder = program.ast().unwrap().to_sql()?;
        builder.to_sql()
    }

    // Literal tests
    #[test_case("42", "42"; "integer literal")]
    #[test_case("0", "0"; "zero literal")]
    #[test_case("999", "999"; "large integer literal")]
    #[test_case("true", "TRUE"; "true literal")]
    #[test_case("false", "FALSE"; "false literal")]
    #[test_case("'hello'", "hello"; "string literal hello")]
    #[test_case("'world'", "world"; "string literal world")]
    #[test_case("null", "NULL"; "null literal")]
    #[test_case("3.14", "3.14"; "float literal pi")]
    #[test_case("0.0", "0"; "zero float literal")]
    fn test_literals(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Identifier tests
    #[test_case("x", "x"; "simple identifier")]
    #[test_case("user_id", "user_id"; "underscore identifier")]
    #[test_case("firstName", "firstName"; "camelCase identifier")]
    fn test_identifiers(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Arithmetic expression tests
    #[test_case("1 + 2", "(1) + (2)"; "addition integers")]
    #[test_case("x + y", "(x) + (y)"; "addition identifiers")]
    #[test_case("10 - 5", "(10) - (5)"; "subtraction integers")]
    #[test_case("a - b", "(a) - (b)"; "subtraction identifiers")]
    #[test_case("3 * 4", "(3) * (4)"; "multiplication integers")]
    #[test_case("x * y", "(x) * (y)"; "multiplication identifiers")]
    #[test_case("8 / 2", "(8) / (2)"; "division integers")]
    #[test_case("a / b", "(a) / (b)"; "division identifiers")]
    #[test_case("10 % 3", "(10) % (3)"; "modulo integers")]
    #[test_case("x % y", "(x) % (y)"; "modulo identifiers")]
    fn test_arithmetic_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Complex arithmetic tests
    #[test_case("1 + 2 * 3", "(1) + ((2) * (3))"; "precedence multiplication")]
    #[test_case("(1 + 2) * 3", "(((1) + (2))) * (3)"; "parentheses precedence")]
    #[test_case("10 - 4 + 2", "((10) - (4)) + (2)"; "left associative addition")]
    #[test_case("20 / 4 / 2", "((20) / (4)) / (2)"; "left associative division")]
    #[test_case("a + b * c - d / e", "((a) + ((b) * (c))) - ((d) / (e))"; "mixed operations")]
    fn test_complex_arithmetic(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Relational expression tests
    #[test_case("x == y", "(x) = (y)"; "equality identifiers")]
    #[test_case("5 == 5", "(5) = (5)"; "equality integers")]
    #[test_case("x != y", "(x) <> (y)"; "inequality identifiers")]
    #[test_case("a != b", "(a) <> (b)"; "inequality different identifiers")]
    #[test_case("x < y", "(x) < (y)"; "less than")]
    #[test_case("10 < 20", "(10) < (20)"; "less than integers")]
    #[test_case("x <= y", "(x) <= (y)"; "less than or equal")]
    #[test_case("5 <= 10", "(5) <= (10)"; "less than or equal integers")]
    #[test_case("x > y", "(x) > (y)"; "greater than")]
    #[test_case("20 > 10", "(20) > (10)"; "greater than integers")]
    #[test_case("x >= y", "(x) >= (y)"; "greater than or equal")]
    #[test_case("10 >= 5", "(10) >= (5)"; "greater than or equal integers")]
    fn test_relational_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Logical expression tests
    #[test_case("true && false", "(TRUE) AND (FALSE)"; "AND boolean literals")]
    #[test_case("x && y", "(x) AND (y)"; "AND identifiers")]
    #[test_case("true || false", "(TRUE) OR (FALSE)"; "OR boolean literals")]
    #[test_case("a || b", "(a) OR (b)"; "OR identifiers")]
    #[test_case("a && b || c", "((a) AND (b)) OR (c)"; "mixed AND OR precedence")]
    #[test_case("a || b && c", "(a) OR ((b) AND (c))"; "mixed OR AND precedence")]
    fn test_logical_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Complex logical combinations
    #[test_case("(a || b) && (c || d)", "(((a) OR (b))) AND (((c) OR (d)))"; "parenthesized logical")]
    #[test_case("!(a && b)", "(!((a) AND (b)))"; "negated logical")]
    #[test_case("x > 5 && y < 10", "((x) > (5)) AND ((y) < (10))"; "mixed relational and logical")]
    #[test_case("age >= 18 || status == 'admin'", "((age) >= (18)) OR ((status) = (admin))"; "age or admin check")]
    fn test_complex_logical_combinations(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Ternary expression tests
    #[test_case("true ? 'yes' : 'no'", "case (TRUE)::bool when true then (yes) else (no)"; "simple ternary")]
    #[test_case("x > 5 ? 10 : 0", "case ((x) > (5))::bool when true then (10) else (0)"; "ternary with comparison")]
    #[test_case("(x > 0 ? true : false) ? 'positive' : 'non-positive'", "case ((case ((x) > (0))::bool when true then (TRUE) else (FALSE)))::bool when true then (positive) else (non-positive)"; "nested ternary condition")]
    #[test_case("age >= 18 ? 'adult' : 'minor'", "case ((age) >= (18))::bool when true then (adult) else (minor)"; "age check ternary")]
    fn test_ternary_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Unary expression tests
    #[test_case("!true", "(!TRUE)"; "negation true")]
    #[test_case("!false", "(!FALSE)"; "negation false")]
    #[test_case("!x", "(!x)"; "negation identifier")]
    #[test_case("-5", "(-5)"; "arithmetic negation integer")]
    #[test_case("-x", "(-x)"; "arithmetic negation identifier")]
    #[test_case("!!true", "(!!TRUE)"; "double negation")]
    #[test_case("--5", "(--5)"; "double arithmetic negation")]
    fn test_unary_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Parenthesized expression tests
    #[test_case("(5)", "(5)"; "simple parentheses integer")]
    #[test_case("(x)", "(x)"; "simple parentheses identifier")]
    #[test_case("((5))", "((5))"; "nested parentheses")]
    #[test_case("(((x + y)))", "((((x) + (y))))"; "deeply nested parentheses")]
    #[test_case("(1 + 2) * 3", "(((1) + (2))) * (3)"; "parentheses changing precedence")]
    #[test_case("1 + (2 * 3)", "(1) + (((2) * (3)))"; "parentheses preserving precedence")]
    fn test_parenthesized_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Complex nested expression tests
    #[test_case("(x + y) > 10 && (a - b) < 5", "((((x) + (y))) > (10)) AND ((((a) - (b))) < (5))"; "complex arithmetic with logic")]
    #[test_case("x > 0 ? x * 2 : x / 2", "case ((x) > (0))::bool when true then ((x) * (2)) else ((x) / (2))"; "ternary with arithmetic")]
    #[test_case("((a + b) * (c - d)) == ((e / f) + (g % h))", "(((((a) + (b))) * (((c) - (d))))) = (((((e) / (f))) + (((g) % (h)))))"; "multiple levels of nesting")]
    #[test_case("(x > 5 ? true : false) && (y < 10 || z == 0)", "((case ((x) > (5))::bool when true then (TRUE) else (FALSE))) AND ((((y) < (10)) OR ((z) = (0))))"; "complex logical with ternary")]
    fn test_complex_nested_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Operator precedence tests
    #[test_case("1 + 2 * 3", "(1) + ((2) * (3))"; "multiplication before addition")]
    #[test_case("2 * 3 + 1", "((2) * (3)) + (1)"; "multiplication before addition reverse")]
    #[test_case("x > 5 && y < 10", "((x) > (5)) AND ((y) < (10))"; "comparison before logical AND")]
    #[test_case("a || b && c", "(a) OR ((b) AND (c))"; "AND before OR")]
    #[test_case("a && b || c && d", "((a) AND (b)) OR ((c) AND (d))"; "mixed AND OR precedence")]
    #[test_case("!x && y", "((!x)) AND (y)"; "unary before binary")]
    #[test_case("-x + y", "((-x)) + (y)"; "unary negation before binary")]
    fn test_operator_precedence(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Real world scenario tests
    #[test_case("age >= 18 && age <= 65", "((age) >= (18)) AND ((age) <= (65))"; "age validation")]
    #[test_case("status == 'active' ? priority : 0", "case ((status) = (active))::bool when true then (priority) else (0)"; "status check with fallback")]
    #[test_case("(value >= min && value <= max) || override", "((((value) >= (min)) AND ((value) <= (max)))) OR (override)"; "range checks")]
    #[test_case("(score > 80 && grade == 'A') || (score > 70 && extra_credit)", "((((score) > (80)) AND ((grade) = (A)))) OR ((((score) > (70)) AND (extra_credit)))"; "complex business logic")]
    #[test_case("user_type == 'admin' ? full_access : (department == 'IT' ? tech_access : basic_access)", "case ((user_type) = (admin))::bool when true then (full_access) else ((case ((department) = (IT))::bool when true then (tech_access) else (basic_access)))"; "nested conditions")]
    fn test_real_world_scenarios(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Edge case tests
    #[test_case("42", "42"; "single integer value")]
    #[test_case("true", "TRUE"; "single boolean value")]
    #[test_case("null", "NULL"; "single null value")]
    #[test_case("((((5))))", "((((5))))"; "multiple parentheses levels")]
    #[test_case("!!!(x > 5)", "(!!!((x) > (5)))"; "complex unary combinations")]
    #[test_case("0 + 0", "(0) + (0)"; "zero addition")]
    #[test_case("0 * x", "(0) * (x)"; "zero multiplication")]
    fn test_edge_cases(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Type casting tests
    #[test_case("int(4)", "4::integer"; "integer casting")]
    #[test_case("int('4')", "4::integer"; "string to integer casting")]
    #[test_case("int()", "NULL::integer"; "empty integer casting")]
    #[test_case("uint(42)", "42::bigint"; "unsigned integer casting")]
    #[test_case("uint('42')", "42::bigint"; "string to unsigned casting")]
    #[test_case("float(3.14)", "3.14::double precision"; "float casting")]
    #[test_case("float('3.14')", "3.14::double precision"; "string to float casting")]
    #[test_case("double(2.71)", "2.71::double precision"; "double casting")]
    #[test_case("string(42)", "42::text"; "integer to string casting")]
    #[test_case("string(true)", "TRUE::text"; "boolean to string casting")]
    #[test_case("bool(1)", "1::boolean"; "integer to boolean casting")]
    #[test_case("bool('true')", "true::boolean"; "string to boolean casting")]
    #[test_case("timestamp('2023-01-01')", "2023-01-01::timestamp"; "timestamp casting")]
    fn test_type_casting(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Type casting with expressions
    #[test_case("int((x + y))", "((x) + (y))::integer"; "casting expression")]
    #[test_case("string((user.name))", "(user.name)::text"; "casting member access")]
    #[test_case("int('5') + 3", "(5::integer) + (3)"; "using cast result in expression")]
    #[test_case("string(42) == 'hello'", "(42::text) = (hello)"; "cast in comparison")]
    #[test_case("int(string(42))", "42::text::integer"; "nested casting")]
    #[test_case("x > 0 ? int(x) : int(0)", "case ((x) > (0))::bool when true then (x::integer) else (0::integer)"; "casting in ternary")]
    fn test_type_casting_with_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Type casting edge cases
    #[test_case("int()", "NULL::integer"; "empty integer cast")]
    #[test_case("string()", "NULL::text"; "empty string cast")]
    #[test_case("bool()", "NULL::boolean"; "empty boolean cast")]
    #[test_case("int(null)", "NULL::integer"; "casting null to integer")]
    #[test_case("float(null)", "NULL::double precision"; "casting null to float")]
    #[test_case("int((5))", "(5)::integer"; "casting parenthesized value")]
    #[test_case("(int(5))", "(5::integer)"; "parenthesized cast")]
    fn test_type_casting_edge_cases(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Non-type function call tests
    #[test_case("max(a, b)", "max(a, b)"; "simple function call")]
    #[test_case("min(1, 2, 3)", "min(1, 2, 3)"; "function with multiple args")]
    #[test_case("someFunction(x, y, z)", "someFunction(x, y, z)"; "custom function call")]
    #[test_case("now()", "now()"; "function with no args")]
    #[test_case("random()", "random()"; "another no-arg function")]
    #[test_case("max(x + 1, y - 1)", "max((x) + (1), (y) - (1))"; "function with expressions")]
    fn test_non_type_function_calls(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Array construction tests
    #[test_case("[]", "ARRAY[]"; "empty array")]
    #[test_case("[1, 2, 3]", "ARRAY[1, 2, 3]"; "simple integer array")]
    #[test_case("['a', 'b', 'c']", "ARRAY[a, b, c]"; "string array")]
    #[test_case("[true, false, true]", "ARRAY[TRUE, FALSE, TRUE]"; "boolean array")]
    #[test_case("[1, 'hello', true]", "ARRAY[1, hello, TRUE]"; "mixed type array")]
    #[test_case("[1, null, 3]", "ARRAY[1, NULL, 3]"; "array with null")]
    #[test_case("[42]", "ARRAY[42]"; "single element array")]
    #[test_case("['single']", "ARRAY[single]"; "single string array")]
    fn test_array_construction(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Array construction with expressions
    #[test_case("[x + 1, y - 1, z * 2]", "ARRAY[(x) + (1), (y) - (1), (z) * (2)]"; "array with arithmetic")]
    #[test_case("[x > 5, y == 'test']", "ARRAY[(x) > (5), (y) = (test)]"; "array with relational")]
    #[test_case("[max(a, b), min(c, d)]", "ARRAY[max(a, b), min(c, d)]"; "array with functions")]
    #[test_case("[int('5'), float('3.14')]", "ARRAY[5::integer, 3.14::double precision]"; "array with casting")]
    #[test_case("[x > 0 ? 1 : 0, y > 0 ? 'pos' : 'neg']", "ARRAY[case ((x) > (0))::bool when true then (1) else (0), case ((y) > (0))::bool when true then (pos) else (neg)]"; "array with ternary")]
    fn test_array_construction_with_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Nested array tests
    #[test_case("[[1, 2], [3, 4]]", "ARRAY[ARRAY[1, 2], ARRAY[3, 4]]"; "array of arrays")]
    #[test_case("[[], [1], [1, 2]]", "ARRAY[ARRAY[], ARRAY[1], ARRAY[1, 2]]"; "mixed nested arrays")]
    fn test_nested_arrays(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Arrays in expressions tests
    #[test_case("[1, 2, 3] == [1, 2, 3]", "(ARRAY[1, 2, 3]) = (ARRAY[1, 2, 3])"; "array equality")]
    #[test_case("x > 0 ? [1, 2] : [3, 4]", "case ((x) > (0))::bool when true then (ARRAY[1, 2]) else (ARRAY[3, 4])"; "array in ternary")]
    #[test_case("[1, 2] != [] && [3, 4] != []", "((ARRAY[1, 2]) <> (ARRAY[])) AND ((ARRAY[3, 4]) <> (ARRAY[]))"; "array logical operations")]
    fn test_arrays_in_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Array edge case tests
    #[test_case("[(1), (2), (3)]", "ARRAY[(1), (2), (3)]"; "array with parenthesized elements")]
    #[test_case("[x + (y * z), string((a + b))]", "ARRAY[(x) + (((y) * (z))), ((a) + (b))::text]"; "array with complex expressions")]
    #[test_case("[!true, -5, !x]", "ARRAY[(!TRUE), (-5), (!x)]"; "array with unary expressions")]
    fn test_array_edge_cases(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // JSON object construction tests
    #[test_case("{}", "'{}'::json"; "empty object")]
    #[test_case("{'key': 'value'}", "json_build_object('key', value)"; "simple string object")]
    #[test_case("{'name': 'John', 'age': 30}", "json_build_object('age', 30, 'name', John)"; "object with mixed types")]
    #[test_case("{'active': true, 'count': 42}", "json_build_object('count', 42, 'active', TRUE)"; "object with boolean and integer")]
    #[test_case("{'data': null, 'valid': false}", "json_build_object('valid', FALSE, 'data', NULL)"; "object with null and boolean")]
    #[test_case("{'nested': {'inner': 'value'}}", "json_build_object('nested', json_build_object('inner', value))"; "nested object")]
    #[test_case("{'x': 1, 'y': 2, 'z': 3}", "json_build_object('z', 3, 'y', 2, 'x', 1)"; "object with multiple fields")]
    #[test_case("{'single': 42}", "json_build_object('single', 42)"; "single field object")]
    fn test_json_object_construction(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // JSON object construction with expressions
    #[test_case("{'sum': x + y, 'diff': a - b}", "json_build_object('diff', (a) - (b), 'sum', (x) + (y))"; "object with arithmetic expressions")]
    #[test_case("{'result': x > 5, 'status': 'ok'}", "json_build_object('status', ok, 'result', (x) > (5))"; "object with comparison")]
    #[test_case("{'value': int('42'), 'text': string(123)}", "json_build_object('text', 123::text, 'value', 42::integer)"; "object with type casting")]
    #[test_case("{'condition': x ? 'yes' : 'no'}", "json_build_object('condition', case (x)::bool when true then (yes) else (no))"; "object with ternary")]
    #[test_case("{'func': max(a, b), 'array': [1, 2, 3]}", "json_build_object('array', ARRAY[1, 2, 3], 'func', max(a, b))"; "object with function and array")]
    fn test_json_object_construction_with_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // JSON member access tests
    #[test_case("user.name", "(user)->'name'"; "simple member access")]
    #[test_case("obj.field", "(obj)->'field'"; "object field access")]
    #[test_case("data.id", "(data)->'id'"; "data field access")]
    #[test_case("person.age", "(person)->'age'"; "person age access")]
    #[test_case("config.enabled", "(config)->'enabled'"; "config field access")]
    fn test_json_member_access(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Nested JSON member access tests
    #[test_case("user.profile.name", "((user)->'profile')->'name'"; "nested member access")]
    #[test_case("data.config.settings", "((data)->'config')->'settings'"; "deeply nested access")]
    #[test_case("obj.inner.deep.value", "(((obj)->'inner')->'deep')->'value'"; "multiple level nesting")]
    #[test_case("person.address.street", "((person)->'address')->'street'"; "address field access")]
    fn test_nested_json_member_access(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // JSON objects and member access in expressions
    #[test_case("{'name': 'John'}.name", "(json_build_object('name', John))->'name'"; "object construction with member access")]
    #[test_case("user.name == 'John'", "((user)->'name') = (John)"; "member access in comparison")]
    #[test_case("obj.count > 5", "((obj)->'count') > (5)"; "member access in relation")]
    #[test_case("data.enabled ? 'yes' : 'no'", "case ((data)->'enabled')::bool when true then (yes) else (no)"; "member access in ternary condition")]
    #[test_case("[user.name, user.email]", "ARRAY[(user)->'name', (user)->'email']"; "member access in array")]
    fn test_json_objects_and_member_access_in_expressions(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // Complex JSON scenarios
    #[test_case("{'users': [{'name': 'John'}, {'name': 'Jane'}]}", "json_build_object('users', ARRAY[json_build_object('name', John), json_build_object('name', Jane)])"; "object with array of objects")]
    #[test_case("{'meta': {'created': timestamp('2023-01-01'), 'version': 1}}", "json_build_object('meta', json_build_object('created', 2023-01-01::timestamp, 'version', 1))"; "nested object with timestamp")]
    #[test_case("config.database.host == 'localhost'", "((config)->'database')->'host' = (localhost)"; "nested member access in comparison")]
    #[test_case("{'computed': x + y, 'nested': {'inner': a * b}}", "json_build_object('computed', (x) + (y), 'nested', json_build_object('inner', (a) * (b)))"; "complex nested expressions")]
    fn test_complex_json_scenarios(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }

    // JSON edge cases
    #[test_case("{'': 'empty_key'}", "json_build_object('', empty_key)"; "empty string key")]
    #[test_case("{'key with spaces': 'value'}", "json_build_object('key with spaces', value)"; "key with spaces")]
    #[test_case("{'123': 'numeric_key'}", "json_build_object('123', numeric_key)"; "numeric string key")]
    #[test_case("obj['dynamic_key']", "(obj)->'dynamic_key'"; "bracket notation member access")]
    fn test_json_edge_cases(cel_expr: &str, expected: &str) {
        assert_eq!(cel_to_sql(cel_expr).unwrap(), expected);
    }
}
