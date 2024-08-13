#![cfg(feature = "eyre")]

use std::borrow::Borrow;

fn err<T: std::fmt::Debug>(r: impl Borrow<eyre::Result<T>>, n: usize) -> String {
    r.borrow()
        .as_ref()
        .unwrap_err()
        .chain()
        .nth(n)
        .unwrap()
        .to_string()
}

#[test]
fn simple_bail() {
    #[context_attr::eyre("Attribute")]
    fn func() -> eyre::Result<()> {
        eyre::bail!("Body");
    }

    let result = func();
    assert_eq!(err(&result, 0), "Attribute");
    assert_eq!(err(&result, 1), "Body");
}

#[test]
fn single_arg() {
    #[context_attr::eyre("Attribute")]
    fn func(n: u32) -> eyre::Result<()> {
        eyre::bail!("Body {n}");
    }

    let result = func(42);
    assert_eq!(err(&result, 1), "Body 42");
}

#[test]
fn arg_used_in_context() {
    #[context_attr::eyre(format!("Attribute {n}"))]
    fn func(n: u32) -> eyre::Result<()> {
        eyre::bail!("Body {n}");
    }

    let result = func(42);
    assert_eq!(err(&result, 0), "Attribute 42");
}

#[test]
fn closure_does_not_move_arg() {
    #[allow(unused)]
    struct A;

    #[context_attr::eyre("Attribute")]
    fn func(_a: A) -> eyre::Result<()> {
        eyre::bail!("Body");
    }
}

// TODO(shelbyd): Consider making this work.
// #[test]
// fn ignored_arg_in_function() {
//     #[context_attr::eyre("Attribute")]
//     fn func(_: u32) -> eyre::Result<()> {
//         eyre::bail!("Body");
//     }
// }

#[test]
fn struct_method() {
    struct A(u32);

    impl A {
        #[context_attr::eyre(format!("Attribute {}", self.0))]
        fn func(&self) -> eyre::Result<()> {
            eyre::bail!("Body {}", self.0);
        }
    }

    let result = A(42).func();
    assert_eq!(err(&result, 0), "Attribute 42");
    assert_eq!(err(&result, 1), "Body 42");
}

#[test]
fn struct_method_mut() {
    struct A(u32);

    impl A {
        #[context_attr::eyre(format!("Attribute {}", self.0))]
        fn func(&mut self) -> eyre::Result<()> {
            self.0 += 1;
            eyre::bail!("Body {}", self.0);
        }
    }

    let result = A(42).func();
    assert_eq!(err(&result, 0), "Attribute 42");
    assert_eq!(err(&result, 1), "Body 43");
}

// TODO(shelbyd): Defer construction of error message only when error actually occurs.
