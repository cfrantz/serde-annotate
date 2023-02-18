#![feature(min_specialization)]
use anyhow::Result;
use serde_annotate::serialize;
use serde_annotate::Annotate;

#[derive(Debug, serde::Serialize, serde::Deserialize, Annotate)]
struct Hello {
    #[annotate(comment = "A greeting")]
    message: String,
}

fn hello() -> Box<dyn Annotate> {
    Box::new(Hello {
        message: "Hello World!".into(),
    })
}

#[test]
fn test_erased_serialization_regular() -> Result<()> {
    let greeting = hello();
    let s = serialize(&*greeting)?.to_json5().to_string();
    assert_eq!(s, "{\n  // A greeting\n  message: \"Hello World!\"\n}");
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NestedHello {
    greeting: Hello,
}

fn nested_hello() -> Box<dyn Annotate> {
    let n = NestedHello {
        greeting: Hello {
            message: "Hola!".into(),
        },
    };
    Box::new(n)
}

#[test]
fn test_erased_serialization_nested() -> Result<()> {
    let greeting = nested_hello();
    let s = serialize(&*greeting)?.to_json5().to_string();
    assert_eq!(
        s,
        "{\n  greeting: {\n    // A greeting\n    message: \"Hola!\"\n  }\n}"
    );
    Ok(())
}
