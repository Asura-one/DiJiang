#[test]
fn test_load_findings_from_file() {
    use std::path::Path;
    let p = Path::new("/Users/cimer/Project/DiJiang/.dijiang");
    let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(p).unwrap();
    let findings = mem.load_findings().unwrap();
    println!("Findings count: {}", findings.len());
    for f in &findings {
        println!("  tags: {:?}, scope: {:?}", f.tags, f.scope);
    }
}

#[test]
fn test_load_learnings_from_file() {
    use std::path::Path;
    let p = Path::new("/Users/cimer/Project/DiJiang/.dijiang");
    let mem = dijiang_mem::ProjectMemory::from_dijiang_dir(p).unwrap();
    let learnings = mem.load_learnings().unwrap();
    println!("Learnings count: {}", learnings.len());
    for l in &learnings {
        println!("  tags: {:?}, scope: {:?}", l.tags, l.scope);
    }
}
