mod interp;
mod types;

pub use interp::Interpreter;
pub use types::*;

#[cfg(test)]
mod test {
    use crate::{types::CelByteCode, CelValue};

    use super::{
        types::{ByteCode, JmpWhen},
        Interpreter,
    };
    use test_case::test_case;

    #[test_case(ByteCode::Add, 7.into())]
    #[test_case(ByteCode::Sub, 1.into())]
    #[test_case(ByteCode::Mul, 12.into())]
    #[test_case(ByteCode::Div, 1.into())]
    #[test_case(ByteCode::Mod, 1.into())]
    #[test_case(ByteCode::Lt, false.into())]
    #[test_case(ByteCode::Le, false.into())]
    #[test_case(ByteCode::Eq, false.into())]
    #[test_case(ByteCode::Ne, true.into())]
    #[test_case(ByteCode::Ge, true.into())]
    #[test_case(ByteCode::Gt, true.into())]
    fn test_interp_ops(op: ByteCode, expected: CelValue) {
        let mut prog =
            CelByteCode::from_vec(vec![ByteCode::Push(4.into()), ByteCode::Push(3.into())]);
        prog.push(op);
        let interp = Interpreter::empty();

        assert!(interp.run_raw(&prog, true).unwrap() == expected);
    }

    #[test]
    fn test_backward_jump_loop_executes() {
        let prog = CelByteCode::from_vec(vec![
            ByteCode::Push(0.into()),
            ByteCode::Dup,
            ByteCode::Push(2.into()),
            ByteCode::Lt,
            ByteCode::JmpCond {
                when: JmpWhen::False,
                dist: 3,
            },
            ByteCode::Push(1.into()),
            ByteCode::Add,
            ByteCode::Jmp(-7),
        ]);

        let interp = Interpreter::empty();
        let result = interp.run_raw(&prog, true).unwrap();

        assert_eq!(result, 2.into());
    }
}
