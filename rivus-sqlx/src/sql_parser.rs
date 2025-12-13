#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use quick_xml::de;
use walkdir::WalkDir;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct IdMapper {
    pub use_generated_keys: Option<String>,
    pub key_column: Option<String>,
}

pub type ContentMap = HashMap<String, HashMap<String, Option<String>>>;
pub type MapperMap = HashMap<String, HashMap<String, IdMapper>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mapper {
    #[serde(rename = "@namespace")]
    namespace: String,
    #[serde(rename = "$value")]
    nodes: Vec<SqlNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SqlNode {
    Sql(SqlItem),
    Select(SqlItem),
    Insert(SqlItem),
    Update(SqlItem),
    Delete(SqlItem),
    #[serde(other)]
    Unknown,
}

impl SqlNode {
    fn into_item(self) -> Option<SqlItem> {
        match self {
            SqlNode::Sql(item) |
            SqlNode::Select(item) |
            SqlNode::Insert(item) |
            SqlNode::Update(item) |
            SqlNode::Delete(item) => Some(item),
            SqlNode::Unknown => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SqlItem {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@useGeneratedKeys")]
    pub use_generated_keys: Option<String>,
    #[serde(rename = "@keyColumn")]
    pub key_column: Option<String>,
    #[serde(rename = "$text")]
    pub content: Option<String>,
}

impl From<&SqlItem> for IdMapper {
    fn from(item: &SqlItem) -> Self {
        Self {
            use_generated_keys: item.use_generated_keys.clone(),
            key_column: item.key_column.clone(),
        }
    }
}

/// 递归读取指定目录及其子目录下的所有 XML 文件，并解析。
pub fn parse_mappers_recursively(
    dir_path: &Path,
    content_map: &mut ContentMap,
    mapper_map: &mut MapperMap,
) -> Result<()> {
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
            process_mapper_file(path, content_map, mapper_map)?;
        }
    }
    Ok(())
}

fn process_mapper_file(
    path: &Path,
    content_map: &mut ContentMap,
    mapper_map: &mut MapperMap,
) -> Result<()> {
    let xml_content = fs::read_to_string(path)
        .with_context(|| format!("读取文件失败: {}", path.display()))?;
    let mapper: Mapper = de::from_str(&xml_content)
        .with_context(|| format!("XML 解析失败: {}", path.display()))?;
    let namespace = mapper.namespace;

    let ns_content_map = content_map.entry(namespace.clone()).or_default();
    let ns_mapper_map = mapper_map.entry(namespace.clone()).or_default();

    for node in mapper.nodes {
        if let Some(item) = node.into_item() {
            let id_mapper = IdMapper::from(&item);
            
            if ns_content_map.insert(item.id.clone(), item.content).is_some() {
                anyhow::bail!(
                    "文件 '{}' 中发现重复的 ID: '{}' (命名空间: '{}')",
                    path.display(),
                    item.id,
                    namespace
                );
            }
            ns_mapper_map.insert(item.id, id_mapper);
        }
    }
    Ok(())
}
