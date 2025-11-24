use serde::Deserialize;
use rivus_yaml::{include_yaml, load_from_file};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub name: String,
    pub sex: i32,
    pub address: String,
}

#[test]
fn test_include_yaml_with_config_struct() {
    // 使用 include_yaml! 宏嵌入 YAML 文件并解析为 Config 结构体
    let config = include_yaml!("config_struct.yaml", Config).unwrap();
    
    
    assert_eq!(config.name, "Alice");
    assert_eq!(config.sex, 1);
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_load_from_file_with_config_struct() {
    // 使用 load_from_file 函数直接加载为 Config 结构体
    let config: Config = load_from_file("tests/config_struct.yaml").unwrap();
    
    assert_eq!(config.name, "Alice");
    assert_eq!(config.sex, 1);
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_config_struct_with_env_vars() {
    // 创建一个包含环境变量的 YAML 文件
    let yaml_content = r#"
name: ${TEST_NAME:Bob}
sex: 2
address: ${TEST_ADDRESS:Guangzhou}
"#;
    
    // 使用 load_from_str 加载
    let config: Config = rivus_yaml::load_from_str(yaml_content).unwrap();
    
    // 如果没有设置环境变量，应该使用默认值
    assert_eq!(config.name, "Bob");
    assert_eq!(config.sex, 2);
    assert_eq!(config.address, "Guangzhou");
}
