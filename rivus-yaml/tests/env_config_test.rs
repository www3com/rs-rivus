use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use rivus_yaml::load_from_file;
use dotenvy;

/// YAML 加载器错误
#[derive(Debug, thiserror::Error)]
pub enum YamlLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    #[error("Invalid variable format: {0}")]
    InvalidVariable(String),
}

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
fn test_env_no_var_config_var_no_default() {
    // 情况1: .env 中没有变量，config 中有变量、没有默认值
    
    // 确保环境变量不存在
    unsafe { env::remove_var("NAME_NO_DEFAULT"); }
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config1.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: ${NAME_NO_DEFAULT}
sex: ${SEX_NO_DEFAULT}
address: ${ADDRESS_NO_DEFAULT}
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 应该使用空字符串作为默认值
    assert_eq!(config.name, "");
    assert_eq!(config.sex, "");
    assert_eq!(config.address, "");
}

#[test]
fn test_env_no_var_config_var_with_default() {
    // 情况2: .env 中没有变量，config 中有变量、有默认值
    
    // 确保环境变量不存在
    unsafe { env::remove_var("NAME_WITH_DEFAULT"); }
    unsafe { env::remove_var("SEX_WITH_DEFAULT"); }
    unsafe { env::remove_var("ADDRESS_WITH_DEFAULT"); }
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config2.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: ${NAME_WITH_DEFAULT:DefaultName}
sex: ${SEX_WITH_DEFAULT:male}
address: ${ADDRESS_WITH_DEFAULT:Beijing}
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 应该使用默认值
    assert_eq!(config.name, "DefaultName");
    assert_eq!(config.sex, "male");
    assert_eq!(config.address, "Beijing");
}

#[test]
fn test_config_no_vars() {
    // 情况3: config 中没有变量（纯文本）
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config3.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: StaticName
sex: female
address: Shanghai
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 应该直接使用配置中的值
    assert_eq!(config.name, "StaticName");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}

#[test]
fn test_env_with_var_config_var_no_default() {
    // 情况4: .env 中有变量，config 中有变量、没有默认值
    
    // 设置环境变量
    unsafe { env::set_var("NAME_ENV_ONLY", "EnvName"); }
    unsafe { env::set_var("SEX_ENV_ONLY", "female"); }
    unsafe { env::set_var("ADDRESS_ENV_ONLY", "Guangzhou"); }
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config4.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: ${NAME_ENV_ONLY}
sex: ${SEX_ENV_ONLY}
address: ${ADDRESS_ENV_ONLY}
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 应该使用环境变量的值
    assert_eq!(config.name, "EnvName");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Guangzhou");
}

#[test]
fn test_env_with_var_config_var_with_default() {
    // 情况5: .env 中有变量，config 中有变量、有默认值
    
    // 设置环境变量
    unsafe { env::set_var("NAME_ENV_OVERRIDE", "EnvOverrideName"); }
    unsafe { env::set_var("SEX_ENV_OVERRIDE", "env_female"); }
    // 不设置 ADDRESS_ENV_OVERRIDE，测试默认值
    unsafe { env::remove_var("ADDRESS_ENV_OVERRIDE"); }
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config5.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: ${NAME_ENV_OVERRIDE:DefaultName}
sex: ${SEX_ENV_OVERRIDE:male}
address: ${ADDRESS_ENV_OVERRIDE:Beijing}
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 环境变量应该覆盖默认值，没有环境变量的使用默认值
    assert_eq!(config.name, "EnvOverrideName");
    assert_eq!(config.sex, "env_female");
    assert_eq!(config.address, "Beijing");
}

#[test]
fn test_env_file_with_vars() {
    // 情况6: 使用 .env 文件中的变量
    
    // 明确加载测试目录下的 .env 文件
    load_env_from_path("tests/env_config_test.env").unwrap();
    
    // 创建临时目录和 YAML 文件
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config6.yaml");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "{}",
        r#"
name: ${ENV_CONFIG_NAME:DefaultName}
sex: ${ENV_CONFIG_SEX:male}
address: ${ENV_CONFIG_ADDRESS:Beijing}
"#
    )
        .unwrap();

    let config: Config = load_from_file(file_path).unwrap();

    // 应该使用 .env 文件中的值
    assert_eq!(config.name, "EnvConfigName");
    assert_eq!(config.sex, "female");
    assert_eq!(config.address, "Shanghai");
}
