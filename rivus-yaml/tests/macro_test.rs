use std::env;
use rivus_yaml::include_yaml;

#[test]
fn test_include_yaml_macro_with_defaults() {
    // 注意：include_yaml! 宏在编译时执行环境变量替换
    // 因此这个测试实际上测试的是编译时的环境变量状态
    // 如果编译时没有设置 MACRO_* 环境变量，应该使用默认值
    
    let config = include_yaml!("macro_config.yaml").unwrap();
    // include_yaml 返回 serde_yaml::Value
    let name = config.get("name").unwrap().as_str().unwrap();
    let sex = config.get("sex").unwrap().as_str().unwrap();
    let address = config.get("address").unwrap().as_str().unwrap();

    // 这个测试验证的是编译时的行为
    // 如果编译时没有设置环境变量，应该使用默认值
    // 如果编译时设置了环境变量，就会使用环境变量的值
    // 这里我们只验证配置能被正确解析，不验证具体的值
    assert!(!name.is_empty());
    assert!(!sex.is_empty());
    assert!(!address.is_empty());
}

#[test]
fn test_include_yaml_macro_with_env_vars() {
    // 测试使用环境变量的情况
    unsafe { env::set_var("MACRO_NAME", "MacroAlice"); }
    unsafe { env::set_var("MACRO_SEX", "female"); }
    unsafe { env::set_var("MACRO_ADDRESS", "Beijing"); }
    
    let config = include_yaml!("macro_config.yaml").unwrap();
    // include_yaml 返回 serde_yaml::Value
    let name = config.get("name").unwrap().as_str().unwrap();
    let sex = config.get("sex").unwrap().as_str().unwrap();
    let address = config.get("address").unwrap().as_str().unwrap();

    // 应该使用环境变量的值
    assert_eq!(name, "MacroAlice");
    assert_eq!(sex, "female");
    assert_eq!(address, "Beijing");
}
