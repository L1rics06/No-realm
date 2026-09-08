//! 备份策略定义

use std::path::PathBuf;

/// 备份策略
#[derive(Debug, Clone)]
pub struct BackupStrategy {
    /// 保留的备份数量
    pub retain_count: usize,

    /// 备份存储位置（None 表示使用默认位置）
    pub backup_dir: Option<PathBuf>,

    /// 是否在备份后验证
    pub verify_after_backup: bool,

    /// 是否包含 files/ 目录的快照
    ///
    /// 注意：files/ 目录可能非常大（几十 GB），通常不需要备份
    pub include_files: bool,
}

impl Default for BackupStrategy {
    fn default() -> Self {
        Self {
            retain_count: 5,
            backup_dir: None,
            verify_after_backup: true,
            include_files: false,
        }
    }
}

impl BackupStrategy {
    /// 创建一个新的备份策略
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置保留数量
    pub fn retain_count(mut self, count: usize) -> Self {
        self.retain_count = count;
        self
    }

    /// 设置备份目录
    pub fn backup_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.backup_dir = Some(dir.into());
        self
    }

    /// 设置是否验证
    pub fn verify_after_backup(mut self, verify: bool) -> Self {
        self.verify_after_backup = verify;
        self
    }

    /// 设置是否包含 files 目录
    pub fn include_files(mut self, include: bool) -> Self {
        self.include_files = include;
        self
    }
}

/// 安全等级预设
pub enum SafetyLevel {
    /// 偏执模式：每次操作都备份，保留所有备份
    Paranoid,

    /// 安全模式：每次操作都备份，保留 5 个备份（默认）
    Safe,

    /// 平衡模式：批量操作只备份一次，保留 3 个备份
    Balanced,

    /// 快速模式：只在会话开始时备份一次
    Fast,
}

impl SafetyLevel {
    /// 转换为备份策略
    pub fn to_strategy(&self) -> BackupStrategy {
        match self {
            Self::Paranoid => BackupStrategy {
                retain_count: usize::MAX, // 保留所有
                backup_dir: None,
                verify_after_backup: true,
                include_files: false,
            },
            Self::Safe => BackupStrategy::default(),
            Self::Balanced => BackupStrategy {
                retain_count: 3,
                backup_dir: None,
                verify_after_backup: true,
                include_files: false,
            },
            Self::Fast => BackupStrategy {
                retain_count: 1,
                backup_dir: None,
                verify_after_backup: false,
                include_files: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strategy() {
        let strategy = BackupStrategy::default();
        assert_eq!(strategy.retain_count, 5);
        assert!(strategy.verify_after_backup);
        assert!(!strategy.include_files);
    }

    #[test]
    fn test_strategy_builder() {
        let strategy = BackupStrategy::new()
            .retain_count(10)
            .verify_after_backup(false)
            .include_files(true);

        assert_eq!(strategy.retain_count, 10);
        assert!(!strategy.verify_after_backup);
        assert!(strategy.include_files);
    }

    #[test]
    fn test_safety_levels() {
        let paranoid = SafetyLevel::Paranoid.to_strategy();
        assert_eq!(paranoid.retain_count, usize::MAX);

        let safe = SafetyLevel::Safe.to_strategy();
        assert_eq!(safe.retain_count, 5);

        let balanced = SafetyLevel::Balanced.to_strategy();
        assert_eq!(balanced.retain_count, 3);

        let fast = SafetyLevel::Fast.to_strategy();
        assert_eq!(fast.retain_count, 1);
    }
}
