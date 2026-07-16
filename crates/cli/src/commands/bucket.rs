use anyhow::Result;
use dijiang_task::{
    bucket_statistics, find_bucket_for_skill, get_bucket_info,
    get_default_buckets, list_bucket_names, list_skills_by_bucket,
};

pub fn cmd_bucket_list(bucket_filter: Option<&str>, skill_filter: Option<&str>) -> Result<()> {
    let config = get_default_buckets();

    // If skill_filter is given, show which bucket a skill belongs to
    if let Some(skill) = skill_filter {
        match find_bucket_for_skill(&config, skill) {
            Some(bucket) => {
                let (label, _) = get_bucket_info(&config, bucket).unwrap_or((bucket, ""));
                println!("{} → {}", skill, label);
            }
            None => {
                println!("{} not found in any bucket", skill);
            }
        }
        return Ok(());
    }

    // If bucket_filter is given, show only that bucket
    if let Some(filter) = bucket_filter {
        let skills = list_skills_by_bucket(&config, filter);
        if skills.is_empty() {
            println!("Bucket '{}' not found or empty.", filter);
            println!("Available buckets: {}", list_bucket_names(&config).join(", "));
            return Ok(());
        }
        let (label, desc) = get_bucket_info(&config, filter).unwrap_or((filter, ""));
        println!("{}: {} — {}", filter, label, desc);
        for skill in &skills {
            println!("  {}", skill);
        }
        return Ok(());
    }

    // Default: show all buckets
    println!("DiJiang Skill Buckets\n");
    for bucket in &config.buckets {
        println!("{} ({}) — {}", bucket.name, bucket.label, bucket.description);
        for skill in &bucket.skills {
            println!("  {}", skill);
        }
        println!();
    }

    Ok(())
}

pub fn cmd_bucket_stats() -> Result<()> {
    let config = get_default_buckets();
    let stats = bucket_statistics(&config);
    let total: usize = stats.iter().map(|(_, c)| c).sum();

    println!("DiJiang Skill Buckets — Statistics");
    println!("  Total skills: {}", total);
    println!("  Total buckets: {}", stats.len());
    println!();
    for (name, count) in &stats {
        let (label, _) = get_bucket_info(&config, name).unwrap_or((name, ""));
        println!("  {:20} {:10} {}", name, count, label);
    }

    Ok(())
}
