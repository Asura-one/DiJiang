use anyhow::Result;
use dijiang_task::BenchmarkResult;
use std::path::PathBuf;

pub fn cmd_bench_list() -> Result<()> {
    let project_root = resolve_project_root();
    let scenarios_dir = project_root
        .join("crates")
        .join("configurator")
        .join("templates")
        .join("benchmarks");
    let bench_dir = scenarios_dir;
    let scenarios = dijiang_task::list_benchmark_scenarios(&bench_dir);
    if scenarios.is_empty() {
        println!("No benchmark scenarios found in {:?}", bench_dir.join("scenarios"));
        return Ok(());
    }

    println!("Available benchmark scenarios:");
    for name in &scenarios {
        println!("  {}", name);
    }
    Ok(())
}

pub fn cmd_bench_run(scenario: Option<&str>, all: bool) -> Result<()> {
    let project_root = resolve_project_root();
    let bench_dir = project_root
        .join("crates")
        .join("configurator")
        .join("templates")
        .join("benchmarks");
    let results_base = project_root.join(".dijiang").join("benchmarks");
    let scenarios = if all {
        dijiang_task::list_benchmark_scenarios(&bench_dir)
    } else if let Some(name) = scenario {
        vec![name.to_string()]
    } else {
        println!("Usage: dijiang bench run <scenario>");
        println!("       dijiang bench run --all");
        cmd_bench_list()?;
        return Ok(());
    };

    let mut all_passed = true;
    let mut results: Vec<BenchmarkResult> = Vec::new();

    for name in &scenarios {
        let scenario = dijiang_task::find_benchmark_scenario(&bench_dir, name);
        match scenario {
            Some(sc) => {
                println!("Running benchmark: {} — {}", sc.name, sc.description);
                let result = dijiang_task::run_benchmark_scenario(&sc, &project_root);
                if result.passed {
                    println!("  ✓ PASSED");
                } else {
                    println!("  ✗ FAILED");
                    all_passed = false;
                }
                for check in &result.checks {
                    println!("    {}", check.message);
                }
                results.push(result);
            }
            None => {
                eprintln!("Benchmark scenario '{}' not found", name);
                all_passed = false;
            }
        }
    }

    // Save results
    // Save results (use .dijiang/benchmarks/results/ for runtime data)
    let results_path = results_base.join("results").join("latest.json");
    if let Some(parent) = results_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&results) {
        let _ = std::fs::write(&results_path, json);
    }

    if all_passed {
        println!("\nAll benchmarks passed.");
    } else {
        eprintln!("\nSome benchmarks failed.");
    }
    Ok(())
}

pub fn cmd_bench_status() -> Result<()> {
    let project_root = resolve_project_root();
    let results_path = project_root
        .join(".dijiang")
        .join("benchmarks")
        .join("results")
        .join("latest.json");

    if !results_path.exists() {
        println!("No benchmark results yet. Run 'dijiang bench run' first.");
        return Ok(());
    }

    match std::fs::read_to_string(&results_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<BenchmarkResult>>(&content) {
                Ok(results) => {
                    for result in &results {
                        let status = if result.passed { "✓ PASSED" } else { "✗ FAILED" };
                        println!("{}: {} at {}", result.scenario, status, result.timestamp);
                        for check in &result.checks {
                            println!("  {}", check.message);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse benchmark results: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read benchmark results: {}", e);
        }
    }
    Ok(())
}

fn resolve_project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = Some(cwd.as_path());
    while let Some(d) = dir {
        if d.join(".dijiang").is_dir() {
            return d.to_path_buf();
        }
        dir = d.parent();
    }
    PathBuf::from(".")
}
