use once_cell::sync::Lazy;
use std::collections::HashMap;

use super::{default_call, join_fragments, SqlFragment, SqlResult};

pub type SqlFunc = Box<dyn Fn(Vec<SqlFragment>) -> SqlResult<SqlFragment> + Send + Sync>;

fn simple(name: &'static str) -> SqlFunc {
    Box::new(move |args| Ok(default_call(name, args)))
}

fn extract(part: &'static str) -> SqlFunc {
    Box::new(move |args| {
        let (parts, params) = join_fragments(args);
        Ok(SqlFragment {
            sql: format!("EXTRACT({} FROM {})", part, parts[0]),
            params,
        })
    })
}

pub static FUNCTIONS: Lazy<HashMap<&'static str, SqlFunc>> = Lazy::new(|| {
    let mut m: HashMap<&'static str, SqlFunc> = HashMap::new();

    // Math
    m.insert("abs", simple("ABS"));
    m.insert("sqrt", simple("SQRT"));
    m.insert("pow", simple("POWER"));
    m.insert("log", simple("LOG"));
    m.insert("ceil", simple("CEIL"));
    m.insert("floor", simple("FLOOR"));
    m.insert("round", simple("ROUND"));
    m.insert("min", simple("LEAST"));
    m.insert("max", simple("GREATEST"));

    // String
    m.insert("toLower", simple("LOWER"));
    m.insert("toUpper", simple("UPPER"));
    m.insert("trim", simple("TRIM"));
    m.insert("trimStart", simple("LTRIM"));
    m.insert("trimEnd", simple("RTRIM"));
    m.insert(
        "contains",
        Box::new(|args| {
            let (parts, params) = join_fragments(args);
            Ok(SqlFragment {
                sql: format!("({} LIKE '%' || {} || '%')", parts[0], parts[1]),
                params,
            })
        }),
    );
    m.insert(
        "startsWith",
        Box::new(|args| {
            let (parts, params) = join_fragments(args);
            Ok(SqlFragment {
                sql: format!("({} LIKE {} || '%')", parts[0], parts[1]),
                params,
            })
        }),
    );
    m.insert(
        "endsWith",
        Box::new(|args| {
            let (parts, params) = join_fragments(args);
            Ok(SqlFragment {
                sql: format!("({} LIKE '%' || {})", parts[0], parts[1]),
                params,
            })
        }),
    );

    // Time
    m.insert("getFullYear", extract("YEAR"));
    m.insert("getMonth", extract("MONTH"));
    m.insert("getDayOfMonth", extract("DAY"));
    m.insert("getDayOfWeek", extract("DOW"));
    m.insert("getHours", extract("HOUR"));
    m.insert("getMinutes", extract("MINUTE"));
    m.insert("getSeconds", extract("SECOND"));

    m
});
