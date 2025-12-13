#[path = "../src/sql_parser.rs"]
mod parser;

use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_parse_mappers_recursively() -> anyhow::Result<()> {
    // 1. 创建临时目录
    let dir = tempdir()?;
    let file_path = dir.path().join("UserMapper.xml");

    // 2. 写入测试 XML 文件
    let xml_content = r#"
<mapper namespace="UserDao">
    <select id="listUsers">
        SELECT * FROM users
    </select>
    <insert id="insertUser" useGeneratedKeys="true" keyColumn="id">
        INSERT INTO users (name) VALUES (#{name})
    </insert>
</mapper>
    "#;
    let mut file = File::create(&file_path)?;
    file.write_all(xml_content.as_bytes())?;

    // 3. 准备接收结果的 Map
    let mut content_map = parser::ContentMap::new();
    let mut mapper_map = parser::MapperMap::new();

    // 4. 调用解析函数
    println!("开始递归解析目录: {:?}", dir.path());
    parser::parse_mappers_recursively(dir.path(), &mut content_map, &mut mapper_map)?;

    // 5. 验证结果
    println!("Content Map keys: {:?}", content_map.keys());
    // 验证 content_map
    let user_dao_content = content_map.get("UserDao").expect("UserDao namespace not found");
    
    println!("UserDao keys: {:?}", user_dao_content.keys());
    assert!(user_dao_content.contains_key("listUsers"));
    assert_eq!(
        user_dao_content.get("listUsers").unwrap().as_deref().unwrap().trim(),
        "SELECT * FROM users"
    );

    assert!(user_dao_content.contains_key("insertUser"));
    assert_eq!(
        user_dao_content.get("insertUser").unwrap().as_deref().unwrap().trim(),
        "INSERT INTO users (name) VALUES (#{name})"
    );

    // 验证 mapper_map
    let user_dao_mapper = mapper_map.get("UserDao").expect("UserDao namespace not found in mapper_map");
    let insert_mapper = user_dao_mapper.get("insertUser").expect("insertUser not found in mapper_map");
    
    println!("Insert Mapper Info: {:?}", insert_mapper);
    assert_eq!(insert_mapper.use_generated_keys.as_deref(), Some("true"));
    assert_eq!(insert_mapper.key_column.as_deref(), Some("id"));

    println!("test_parse_mappers_recursively passed!");
    Ok(())
}

#[test]
fn test_parse_invalid_xml() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("Invalid.xml");
    
    println!("创建无效 XML 文件: {:?}", file_path);
    // 写入无效 XML
    let xml_content = "<mapper> unclosed tag";
    let mut file = File::create(&file_path)?;
    file.write_all(xml_content.as_bytes())?;

    let mut content_map = parser::ContentMap::new();
    let mut mapper_map = parser::MapperMap::new();

    // 应该返回错误
    let result = parser::parse_mappers_recursively(dir.path(), &mut content_map, &mut mapper_map);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    println!("捕获到预期错误: {}", err_msg);
    
    assert!(err_msg.contains("XML 解析失败"));
    assert!(err_msg.contains("Invalid.xml"));

    println!("test_parse_invalid_xml passed!");
    Ok(())
}

#[test]
fn test_duplicate_id_error() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("Duplicate.xml");
    
    println!("创建包含重复 ID 的 XML 文件: {:?}", file_path);
    // 写入包含重复 ID 的 XML
    let xml_content = r#"
<mapper namespace="TestDao">
    <select id="dup">SQL1</select>
    <update id="dup">SQL2</update>
</mapper>
    "#;
    let mut file = File::create(&file_path)?;
    file.write_all(xml_content.as_bytes())?;

    let mut content_map = parser::ContentMap::new();
    let mut mapper_map = parser::MapperMap::new();

    // 应该返回错误
    let result = parser::parse_mappers_recursively(dir.path(), &mut content_map, &mut mapper_map);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    println!("捕获到预期重复 ID 错误: {}", err_msg);

    assert!(err_msg.contains("重复的 ID"));
    assert!(err_msg.contains("dup"));

    println!("test_duplicate_id_error passed!");
    Ok(())
}
