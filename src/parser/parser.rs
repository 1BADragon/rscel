use super::{syntax_error::SyntaxError, tokenizer::Tokenizer};

pub struct CelParser<'l> {
    tokenizer: Tokenizer<'l>,
}

impl<'l> CelParser<'l> {
    pub fn with_input(input_str: &'l str) -> CelParser<'l> {
        CelParser {
            tokenizer: Tokenizer::with_input(input_str),
        }
    }

    pub fn parse(&self) -> Result<(), SyntaxError> {
        Ok(())
    }
}

// #[cfg(test)]
// mod test {

//     use test_case::test_case;

//     #[test_case("3+1"; "addition")]
//     #[test_case("(1+foo) / 23"; "with literal")]
//     #[test_case("(true || false) + 23"; "with boolean")]
//     #[test_case("foo.bar"; "member access")]
//     #[test_case("foo[3]"; "list access")]
//     #[test_case("foo.bar()"; "member call")]
//     #[test_case("foo()"; "empty function call")]
//     #[test_case("foo(3)"; "function call")]
//     #[test_case("1"; "just 1")]
//     #[test_case("foo"; "an ident")]
//     #[test_case("foo.bar.baz"; "deep member access")]
//     #[test_case("--foo"; "double neg")]
//     #[test_case("foo || true"; "or")]
//     #[test_case("int(foo.bar && foo.baz) + 4 - (8 * 7)"; "complex")]
//     #[test_case("true ? 3 : 1"; "ternary")]
//     fn test_parser(input: &str) {
//         let expr: Result<Expr, parsel::Error> = parsel::parse_str(input);

//         match expr {
//             Ok(_) => {}
//             Err(err) => {
//                 let span = err.span();

//                 panic!(
//                     "Error from column {} to column {}",
//                     span.start().column,
//                     span.end().column
//                 );
//             }
//         };
//     }
// }
