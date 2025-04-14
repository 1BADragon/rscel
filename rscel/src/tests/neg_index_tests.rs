use crate::{BindContext, CelContext};

#[test]
fn test_neg_index() {
    let mut ctx = CelContext::new();
    let bindings = BindContext::new();

    ctx.add_program_str("test1", "[1,2,3][-1]")
        .expect("Failed to compile program");
    ctx.add_program_str("test2", "[1,2,3][-2]")
        .expect("Failed to compile program");

    if cfg!(feature = "neg_index") {
        assert_eq!(ctx.exec("test1", &bindings).unwrap(), 3.into());
        assert_eq!(ctx.exec("test2", &bindings).unwrap(), 2.into());
    } else {
        assert!(ctx.exec("test1", &bindings).is_err());
        assert!(ctx.exec("test2", &bindings).is_err());
    }
}
