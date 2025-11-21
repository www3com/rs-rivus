// src/lib.rs
use anyhow::Result;
use std::fs::{self, File};
use std::io;
use std::path::{Path};
use zip::read::ZipArchive;

/// 从zip文件中解压内容到目标目录
///
/// # 参数
///
/// * `zip_path` - zip文件的路径
/// * `output_dir` - 解压目标目录
///
/// # 返回值
///
/// * `Result<()>` - 成功返回 Ok(()), 失败返回错误
pub fn extract_zip<P: AsRef<Path>>(zip_path: P, output_dir: P) -> Result<()> {
    // 确保输出目录存在
    fs::create_dir_all(&output_dir)?;

    // 打开zip文件
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // 遍历并解压所有文件
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = output_dir.as_ref().join(outpath);

        // 创建所需的目录结构
        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // 处理文件或目录
        if file.name().ends_with('/') {
            // 这是一个目录
            fs::create_dir_all(&outpath)?;
        } else {
            // 这是一个文件
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // 设置文件权限（仅限 Unix 平台）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// 从zip文件中提取特定的文件到目标目录
///
/// # 参数
///
/// * `zip_path` - zip文件的路径
/// * `file_path` - 要提取的文件在zip中的路径
/// * `output_path` - 输出路径
///
/// # 返回值
///
/// * `Result<()>` - 成功返回 Ok(()), 失败返回错误
pub fn extract_file<P: AsRef<Path>>(zip_path: P, file_path: &str, output_path: P) -> Result<()> {
    // 打开zip文件
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // 提取特定文件
    let mut zip_file = archive.by_name(file_path)?;
    let mut output_file = File::create(output_path)?;
    io::copy(&mut zip_file, &mut output_file)?;

    Ok(())
}

/// 列出zip文件中所有文件的路径
///
/// # 参数
///
/// * `zip_path` - zip文件的路径
///
/// # 返回值
///
/// * `Result<Vec<String>>` - 成功返回文件路径列表, 失败返回错误
pub fn list_files<P: AsRef<Path>>(zip_path: P) -> Result<Vec<String>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut file_names = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        file_names.push(file.name().to_string());
    }

    Ok(file_names)
}

/// 判断zip文件是否包含指定路径的文件
///
/// # 参数
///
/// * `zip_path` - zip文件的路径
/// * `file_path` - 要检查的文件在zip中的路径
///
/// # 返回值
///
/// * `Result<bool>` - 成功返回布尔值(是否包含), 失败返回错误
pub fn contains_file<P: AsRef<Path>>(zip_path: P, file_path: &str) -> Result<bool> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    Ok(archive.by_name(file_path).is_ok())
}

/// 验证zip文件的完整性
///
/// # 参数
///
/// * `zip_path` - zip文件的路径
///
/// # 返回值
///
/// * `Result<()>` - 成功返回 Ok(()), 文件无效或损坏则返回错误
pub fn validate_zip<P: AsRef<Path>>(zip_path: P) -> Result<()> {
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // 尝试遍历所有文件以验证zip结构
    for i in 0..archive.len() {
        let _ = archive.by_index(i)?;
    }

    Ok(())
}