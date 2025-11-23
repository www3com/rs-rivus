use std::env;
use std::path::Path;
use rivus_yaml::{load_from_file, load_from_str, YamlLoaderError};
use dotenvy;

/// 从指定路径加载 .env 文件
/// 这个函数只在测试中使用
fn load_env_from_path<P: AsRef<Path>>(path: P) -> Result<(), YamlLoaderError> {
    dotenvy::from_path(path).map_err(|e| YamlLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    Ok(())
}
#[derive(Debug, serde::Deserialize, PartialEq)]
struct Config {
    name: String,
    sex: String,
    address: String,
}

#[test]
fn test_file_not_found() {
    // 测试文件不存在的情况
    let result = load_from_file::<Config, _>("nonexistent_file.yaml");
    assert!(result.is_err());
    
    if let Err(YamlLoaderError::Io(_)) = result {
        // 期望的IO错误
    } else {
        panic!("Expected IO error for non-existent file");
    }
}

#[test]
fn test_invalid_yaml_format() {
    // 测试无效的YAML格式 - 使用tab字符（YAML不允许tab）
    let invalid_yaml = "\nname: ${NAME}\n\tsex: ${SEX:female}\naddress: ${ADDRESS:Shanghai}\n";

    let result = load_from_str::<Config>(invalid_yaml);
    assert!(result.is_err());
    
    if let Err(YamlLoaderError::YamlParse(_)) = result {
        // 期望的YAML解析错误
    } else {
        panic!("Expected YAML parse error for invalid format");
    }
}

#[test]
fn test_missing_env_var_without_default() {
    // 测试没有默认值的环境变量缺失
    unsafe { env::remove_var("MISSING_VAR"); }
    
    let yaml_str = r#"
name: ${MISSING_VAR}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let config: Config = load_from_str(yaml_str).unwrap();
    
    // 应该使用空字符串作为默认值
    assert_eq!(config.name, "");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_env_file_not_found() {
    // 测试不存在的.env文件
    let result = load_env_from_path("nonexistent.env");
    assert!(result.is_err());
    
    if let Err(YamlLoaderError::Io(_)) = result {
        // 期望的IO错误
    } else {
        panic!("Expected IO error for non-existent .env file");
    }
}

#[test]
fn test_special_characters_in_vars() {
    // 测试特殊字符在环境变量中的处理
    unsafe { env::set_var("SPECIAL_VAR", "value with spaces & symbols@#$"); }
    
    let yaml_str = r#"
name: ${SPECIAL_VAR}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let config: Config = load_from_str(yaml_str).unwrap();
    
    assert_eq!(config.name, "value with spaces & symbols@#$");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_empty_default_value() {
    // 测试空字符串作为默认值
    unsafe { env::remove_var("EMPTY_DEFAULT"); }
    
    let yaml_str = r#"
name: ${EMPTY_DEFAULT:}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let config: Config = load_from_str(yaml_str).unwrap();
    
    assert_eq!(config.name, "");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_complex_default_value() {
    // 测试包含特殊字符的默认值
    unsafe { env::remove_var("COMPLEX_DEFAULT"); }
    
    let yaml_str = r#"
name: ${COMPLEX_DEFAULT:default with spaces & symbols@#$}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let config: Config = load_from_str(yaml_str).unwrap();
    
    assert_eq!(config.name, "default with spaces & symbols@#$");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_multiple_vars_in_one_line() {
    // 测试一行中有多个变量
    unsafe { env::set_var("FIRST_VAR", "first"); }
    unsafe { env::set_var("SECOND_VAR", "second"); }
    
    let yaml_str = r#"
name: ${FIRST_VAR} and ${SECOND_VAR}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let config: Config = load_from_str(yaml_str).unwrap();
    
    assert_eq!(config.name, "first and second");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_nested_var_syntax() {
    // 测试变量语法嵌套（虽然不支持，但应该优雅处理）
    let yaml_str = r#"
name: ${OUTER_${INNER_VAR}}
sex: ${SEX:female}
address: ${ADDRESS:Shanghai}
"#;

    let result = load_from_str::<Config>(yaml_str);
    // 这种嵌套语法应该会导致解析错误或使用空字符串
    assert!(result.is_ok()); // 或者根据实际实现可能是错误
}
