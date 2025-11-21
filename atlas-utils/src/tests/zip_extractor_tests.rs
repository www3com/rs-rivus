// tests/integration_tests.rs

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;
    use zip::{write::FileOptions, ZipWriter};
    use crate::zip_extract;

    // 测试辅助函数：创建测试用ZIP文件
    fn create_test_zip(dir: &Path) -> Result<String> {
        let zip_path = dir.join("test.zip");
        let file = File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);

        // 添加一些测试文件
        let options:FileOptions<()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        // 添加文本文件
        zip.start_file("test.txt", options)?;
        zip.write_all(b"This is a test file content")?;

        // 添加子目录中的文件
        zip.start_file("subdir/nested.txt", options)?;
        zip.write_all(b"This is a nested file")?;

        // 添加空目录
        zip.add_directory("empty_dir", options)?;

        zip.finish()?;

        Ok(zip_path.to_string_lossy().to_string())
    }

    #[test]
    fn test_extract_zip() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 创建输出目录
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&output_dir)?;

        // 测试解压功能
        zip_extract::extract_zip(&zip_path, &output_dir.to_string_lossy().to_string())?;

        // 验证文件是否被正确解压
        assert!(output_dir.join("test.txt").exists());
        assert!(output_dir.join("subdir").exists());
        assert!(output_dir.join("subdir/nested.txt").exists());
        assert!(output_dir.join("empty_dir").exists());

        // 检查文件内容
        let content = fs::read_to_string(output_dir.join("test.txt"))?;
        assert_eq!(content, "This is a test file content");

        Ok(())
    }

    #[test]
    fn test_extract_file() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 测试提取单个文件
        let output_file = temp_dir.path().join("extracted.txt");
        let output_path = output_file.to_string_lossy().to_string();
        zip_extract::extract_file(
            &zip_path, 
            "test.txt", 
            &output_path
        )?;

        // 验证文件是否被正确提取
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file)?;
        assert_eq!(content, "This is a test file content");

        // 测试提取不存在的文件（应该返回错误）
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");
        let nonexistent_path_str = nonexistent_path.to_string_lossy().to_string();
        let result = zip_extract::extract_file(
            &zip_path, 
            "nonexistent.txt", 
            &nonexistent_path_str
        );
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_list_files() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 获取文件列表
        let files = zip_extract::list_files(&zip_path)?;

        // 验证列表是否正确
        // assert_eq!(files.len(), 4); // 3个文件+1个目录
        assert!(files.contains(&"test.txt".to_string()));
        assert!(files.contains(&"subdir/nested.txt".to_string()));
        assert!(files.contains(&"empty_dir/".to_string()));

        Ok(())
    }

    #[test]
    fn test_contains_file() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 测试文件存在
        assert!(zip_extract::contains_file(&zip_path, "test.txt")?);
        assert!(zip_extract::contains_file(&zip_path, "subdir/nested.txt")?);

        // 测试文件不存在
        assert!(!zip_extract::contains_file(&zip_path, "nonexistent.txt")?);

        Ok(())
    }

    #[test]
    fn test_validate_zip() -> Result<()> {
        // 创建临时目录
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 验证有效的ZIP文件
        assert!(zip_extract::validate_zip(&zip_path).is_ok());

        // 创建一个无效的ZIP文件
        let invalid_zip = temp_dir.path().join("invalid.zip");
        let mut file = File::create(&invalid_zip)?;
        file.write_all(b"This is not a valid ZIP file")?;

        // 验证无效的ZIP文件（应该返回错误）
        assert!(zip_extract::validate_zip(&invalid_zip).is_err());

        Ok(())
    }

    #[test]
    fn test_error_handling() -> Result<()> {
        // 测试不存在的文件
        let result = zip_extract::extract_zip("nonexistent.zip", "output");
        assert!(result.is_err());

        // 测试无效的输出路径
        let temp_dir = TempDir::new()?;
        let zip_path = create_test_zip(temp_dir.path())?;

        // 创建一个文件来阻止目录创建
        let blocking_file = temp_dir.path().join("blocking_file");
        File::create(&blocking_file)?;

        // 尝试将这个文件作为输出目录（应该失败）
        let blocking_path = blocking_file.to_string_lossy().to_string();
        let result = zip_extract::extract_zip(
            &zip_path, 
            &blocking_path
        );
        assert!(result.is_err());

        Ok(())
    }
}