/// Bucket 组织
///
/// 从 Skills 项目的桶分类借鉴而来，将 DiJiang 的 33 个 `dj-*` skills
/// 分为 core / specialized / extended / internal 四个桶。
///
/// 桶定义在此模块中静态声明（不是从 .dijiang/ 产物目录加载）。

/// 桶配置（顶层结构）
#[derive(Debug, Clone)]
pub struct BucketConfig {
    pub version: u32,
    pub buckets: Vec<BucketCategory>,
}

/// 单个桶分类
#[derive(Debug, Clone)]
pub struct BucketCategory {
    pub name: String,
    pub label: String,
    pub description: String,
    pub skills: Vec<String>,
}

/// 获取内置桶定义（数据在 Rust 源码中，不在 .dijiang/ 中）
pub fn get_default_buckets() -> BucketConfig {
    BucketConfig {
        version: 1,
        buckets: vec![
            BucketCategory {
                name: "core".into(),
                label: "核心技能".into(),
                description: "经过充分打磨的日常必备技能".into(),
                skills: vec![
                    "dj-grill".into(),
                    "dj-implement".into(),
                    "dj-check".into(),
                    "dj-split".into(),
                    "dj-output".into(),
                    "dj-ponytail".into(),
                    "dj-dispatch".into(),
                    "dj-tdd".into(),
                ],
            },
            BucketCategory {
                name: "specialized".into(),
                label: "专业技能".into(),
                description: "特定场景下可靠可用的专业工具".into(),
                skills: vec![
                    "dj-hunt".into(),
                    "dj-audit".into(),
                    "dj-review".into(),
                    "dj-debt".into(),
                    "dj-health".into(),
                    "dj-research".into(),
                    "dj-pattern".into(),
                    "dj-prototype".into(),
                    "dj-reason".into(),
                    "dj-domain-modeling".into(),
                    "dj-codebase-design".into(),
                ],
            },
            BucketCategory {
                name: "extended".into(),
                label: "扩展技能".into(),
                description: "小眾或实验性技能，按需使用".into(),
                skills: vec![
                    "dj-remix".into(),
                    "dj-design".into(),
                    "dj-script".into(),
                    "dj-write".into(),
                    "dj-karpathy".into(),
                    "dj-absorb".into(),
                    "dj-meta".into(),
                    "dj-ask".into(),
                ],
            },
            BucketCategory {
                name: "internal".into(),
                label: "内部技能".into(),
                description: "DiJiang 系统内部技能，用户通常不直接调用".into(),
                skills: vec![
                    "dijiang-start".into(),
                    "dijiang-continue".into(),
                    "dijiang-finish-work".into(),
                    "dj-git-guardrails".into(),
                    "dj-handoff".into(),
                ],
            },
        ],
    }
}

/// 列出所有桶名
pub fn list_bucket_names(config: &BucketConfig) -> Vec<String> {
    config.buckets.iter().map(|b| b.name.clone()).collect()
}

/// 按桶名列出技能
pub fn list_skills_by_bucket(config: &BucketConfig, bucket_name: &str) -> Vec<String> {
    config
        .buckets
        .iter()
        .find(|b| b.name == bucket_name)
        .map(|b| b.skills.clone())
        .unwrap_or_default()
}

/// 查找技能所属的桶
pub fn find_bucket_for_skill<'a>(config: &'a BucketConfig, skill_name: &str) -> Option<&'a str> {
    config
        .buckets
        .iter()
        .find(|b| b.skills.iter().any(|s| s == skill_name))
        .map(|b| b.name.as_str())
}

/// 获取桶的描述信息
pub fn get_bucket_info<'a>(config: &'a BucketConfig, bucket_name: &str) -> Option<(&'a str, &'a str)> {
    config
        .buckets
        .iter()
        .find(|b| b.name == bucket_name)
        .map(|b| (b.label.as_str(), b.description.as_str()))
}

/// 获取每个桶的技能数量统计
pub fn bucket_statistics(config: &BucketConfig) -> Vec<(&str, usize)> {
    config
        .buckets
        .iter()
        .map(|b| (b.name.as_str(), b.skills.len()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> BucketConfig {
        get_default_buckets()
    }

    #[test]
    fn default_buckets_are_defined() {
        let config = test_config();
        assert_eq!(config.version, 1, "Expected version 1");
        assert_eq!(config.buckets.len(), 4, "Expected 4 buckets");
    }

    #[test]
    fn all_core_skills_have_correct_names() {
        let config = test_config();
        let core = config.buckets.iter().find(|b| b.name == "core").expect("core bucket");
        assert!(core.skills.contains(&"dj-grill".to_string()));
        assert!(core.skills.contains(&"dj-implement".to_string()));
        assert!(core.skills.contains(&"dj-check".to_string()));
    }

    #[test]
    fn each_bucket_has_at_least_one_skill() {
        let config = test_config();
        for bucket in &config.buckets {
            assert!(!bucket.skills.is_empty(), "Bucket '{}' has no skills", bucket.name);
        }
    }

    #[test]
    fn find_bucket_for_skill_works() {
        let config = test_config();
        assert_eq!(find_bucket_for_skill(&config, "dj-grill"), Some("core"));
        assert_eq!(find_bucket_for_skill(&config, "dj-hunt"), Some("specialized"));
        assert_eq!(find_bucket_for_skill(&config, "dj-remix"), Some("extended"));
        assert_eq!(find_bucket_for_skill(&config, "dijiang-start"), Some("internal"));
        assert_eq!(find_bucket_for_skill(&config, "nonexistent"), None);
    }

    #[test]
    fn bucket_statistics_is_non_zero() {
        let config = test_config();
        let stats = bucket_statistics(&config);
        let total: usize = stats.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 32, "Expected 32 total skills across all buckets");
    }

    #[test]
    fn list_bucket_names_returns_all_buckets() {
        let config = test_config();
        let names = list_bucket_names(&config);
        assert!(names.contains(&"core".to_string()));
        assert!(names.contains(&"specialized".to_string()));
        assert!(names.contains(&"extended".to_string()));
        assert!(names.contains(&"internal".to_string()));
    }

    #[test]
    fn list_skills_by_bucket_returns_correct_skills() {
        let config = test_config();
        let core_skills = list_skills_by_bucket(&config, "core");
        assert!(core_skills.contains(&"dj-grill".to_string()));
        assert!(core_skills.contains(&"dj-tdd".to_string()));
        assert_eq!(core_skills.len(), 8);

        let bad = list_skills_by_bucket(&config, "nonexistent");
        assert!(bad.is_empty());
    }

    #[test]
    fn get_bucket_info_returns_label_and_description() {
        let config = test_config();
        let info = get_bucket_info(&config, "core");
        assert!(info.is_some());
        let (label, desc) = info.unwrap();
        assert_eq!(label, "核心技能");
        assert!(!desc.is_empty());

        let none = get_bucket_info(&config, "nonexistent");
        assert!(none.is_none());
    }

    #[test]
    fn bucket_statistics_counts_match() {
        let config = test_config();
        let stats = bucket_statistics(&config);
        assert!(stats.iter().any(|(n, c)| *n == "core" && *c == 8));
        assert!(stats.iter().any(|(n, c)| *n == "specialized" && *c == 11));
        assert!(stats.iter().any(|(n, c)| *n == "extended" && *c == 8));
        assert!(stats.iter().any(|(n, c)| *n == "internal" && *c == 5));
    }
}
