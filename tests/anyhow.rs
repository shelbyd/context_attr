#![cfg(feature = "anyhow")]

use std::borrow::Borrow;

fn err<T: std::fmt::Debug>(r: impl Borrow<anyhow::Result<T>>, n: usize) -> String {
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
    #[context_attr::anyhow("Attribute")]
    fn func() -> anyhow::Result<()> {
        anyhow::bail!("Body");
    }

    let result = func();
    assert_eq!(err(&result, 0), "Attribute");
    assert_eq!(err(&result, 1), "Body");
}
