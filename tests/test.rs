use serde::de::{Deserialize, Deserializer, SeqAccess};
use serde_untagged::UntaggedEnumVisitor;

#[test]
fn test_string_or_array_string() {
    #[derive(PartialEq, Debug)]
    enum Value {
        Single(String),
        Multiple(Vec<String>),
    }

    impl<'de> Deserialize<'de> for Value {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            UntaggedEnumVisitor::new()
                .string(|string| Ok(Value::Single(string.to_owned())))
                .seq(|mut seq| {
                    let mut array = Vec::new();
                    while let Some(element) = seq.next_element()? {
                        array.push(element);
                    }
                    Ok(Value::Multiple(array))
                })
                .deserialize(deserializer)
        }
    }

    let j = r#" "..." "#;
    let v: Value = serde_json::from_str(j).unwrap();
    assert_eq!(v, Value::Single("...".to_owned()));

    let j = r#" ["a","z"] "#;
    let v: Value = serde_json::from_str(j).unwrap();
    assert_eq!(v, Value::Multiple(vec!["a".to_owned(), "z".to_owned()]));
}

#[test]
fn test_borrowed() {
    #[derive(PartialEq, Debug)]
    enum Value<'de> {
        Single(&'de str),
        Multiple(Vec<&'de str>),
    }

    impl<'de> Deserialize<'de> for Value<'de> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            UntaggedEnumVisitor::new()
                .borrowed_str(|string| Ok(Value::Single(string)))
                .seq(|mut seq| {
                    let mut array = Vec::new();
                    while let Some(element) = seq.next_element()? {
                        array.push(element);
                    }
                    Ok(Value::Multiple(array))
                })
                .deserialize(deserializer)
        }
    }

    let j = &r#" "..." "#.to_owned();
    let v: Value = serde_json::from_str(j).unwrap();
    assert_eq!(v, Value::Single("..."));

    let j = &r#" ["a","z"] "#.to_owned();
    let v: Value = serde_json::from_str(j).unwrap();
    assert_eq!(v, Value::Multiple(vec!["a", "z"]));
}
