use timecat::*;

fn test_serde<T>(data: T) -> std::result::Result<(), Box<dyn Error>>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + PartialEq,
{
    let json = serde_json::to_string(&data)?;
    let de_data: T = serde_json::from_str(&json)?;
    assert_eq!(data, de_data);
    Ok(())
}

macro_rules! test_serde_wrapper {
    ($func_name: ident, $data: expr) => {
        #[test]
        fn $func_name() -> std::result::Result<(), Box<dyn Error>> {
            test_serde(SerdeWrapper::new($data))
        }
    };
}

macro_rules! test_serde_wrapper_empty_array {
    ($func_name: ident, $type: ty) => {
        #[test]
        fn $func_name() -> std::result::Result<(), Box<dyn Error>> {
            let empty_array: [$type; 0] = [];
            test_serde(SerdeWrapper::new(empty_array))
        }
    };
}

test_serde_wrapper!(serde_wrapper_test_1, [1, 2, 3, 4, 5]);
test_serde_wrapper!(serde_wrapper_test_2, [1, 2, 3, 4, 5].map(|i| i.to_string()));

test_serde_wrapper_empty_array!(serde_wrapper_empty_array_test_1, i32);
test_serde_wrapper_empty_array!(serde_wrapper_empty_array_test_2, String);
test_serde_wrapper_empty_array!(serde_wrapper_empty_array_test_3, Vec<String>);

#[test]
fn serde_wrapper_empty_array_test_str() -> std::result::Result<(), Box<dyn Error>> {
    let data: [&str; 0] = [];
    let json = serde_json::to_string(&data)?;
    let de_data: [&str; 0] = serde_json::from_str(&json)?;
    assert_eq!(data, de_data);
    Ok(())
}
