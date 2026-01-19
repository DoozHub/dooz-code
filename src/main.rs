//! Dooz-Code CLI
//!
//! Command-line interface for the autonomous execution engine.
//!
//! Usage:
//!   dooz-code execute --package <path> --repo <path>
//!   dooz-code analyze --repo <path>
//!   dooz-code plan --package <path> --repo <path>

#[cfg(feature = "binary")]
use clap::{Parser, Subcommand};

#[cfg(feature = "binary")]
use dooz_code::{
    execute, WorkPackage, RepoContext, VERSION,
    analyzer::RepoAnalyzer,
    planner::ImplementationPlanner,
};

#[cfg(feature = "binary")]
use std::path::PathBuf;

#[cfg(feature = "binary")]
#[derive(Parser)]
#[command(name = "dooz-code")]
#[command(version = VERSION)]
#[command(about = "An autonomous coder that belongs inside a company")]
#[command(long_about = "Dooz-Code executes approved work packages into production code.\n\nIt does NOT:\n- Complete code on demand\n- Chat with users\n- Expand scope\n- Make decisions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "binary")]
#[derive(Subcommand)]
enum Commands {
    /// Execute an approved work package
    Execute {
        /// Path to work package file (JSON or YAML)
        #[arg(short, long)]
        package: PathBuf,

        /// Path to repository root
        #[arg(short, long)]
        repo: PathBuf,

        /// Dry run (plan only, no file changes)
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Analyze a repository's context
    Analyze {
        /// Path to repository root
        #[arg(short, long)]
        repo: PathBuf,

        /// Output format (json, yaml, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,
    },

    /// Plan implementation steps without executing
    Plan {
        /// Path to work package file (JSON or YAML)
        #[arg(short, long)]
        package: PathBuf,

        /// Path to repository root
        #[arg(short, long)]
        repo: PathBuf,

        /// Output format (json, yaml, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,
    },

    /// Validate work package format
    Validate {
        /// Path to work package file (JSON or YAML)
        #[arg(short, long)]
        package: PathBuf,
    },

    /// Show version and configuration
    Info,
}

#[cfg(feature = "binary")]
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Execute { package, repo, dry_run } => {
            cmd_execute(&package, &repo, dry_run);
        }
        Commands::Analyze { repo, format } => {
            cmd_analyze(&repo, &format);
        }
        Commands::Plan { package, repo, format } => {
            cmd_plan(&package, &repo, &format);
        }
        Commands::Validate { package } => {
            cmd_validate(&package);
        }
        Commands::Info => {
            cmd_info();
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_execute(package: &PathBuf, repo: &PathBuf, dry_run: bool) {
    println!("Dooz-Code v{}", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Load work package
    println!("Loading work package: {}", package.display());
    let work_package = match load_work_package(package) {
        Ok(wp) => wp,
        Err(e) => {
            eprintln!("Error: Failed to load work package: {}", e);
            std::process::exit(1);
        }
    };

    // Load repository context
    println!("Analyzing repository: {}", repo.display());
    let context = match RepoContext::from_path(repo) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: Failed to analyze repository: {}", e);
            std::process::exit(1);
        }
    };

    if dry_run {
        println!("Mode: DRY RUN (no files will be modified)\n");
        
        // Run planning only
        let analyzed = match RepoAnalyzer::new().analyze(&context) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Error: Analysis failed: {}", e);
                std::process::exit(1);
            }
        };

        let planner = ImplementationPlanner::new();
        match planner.plan(&work_package, &analyzed) {
            Ok(plan) => {
                println!("Plan Summary");
                println!("├── Steps: {}", plan.steps.len());
                println!("├── Test Plan: {} tests", plan.test_plan.test_files.len());
                println!("└── Complexity: {}/10\n", plan.complexity.overall);
                
                println!("Steps:");
                for (i, step) in plan.steps.iter().enumerate() {
                    println!("  {}. [{}] {}", i + 1, format_step_type(&step.step_type), step.description);
                    if !step.target.is_empty() {
                        println!("     └── Target: {}", step.target);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Planning failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        println!("Mode: EXECUTE\n");
        
        // Execute
        match execute(&work_package, &context) {
            Ok(result) => {
                println!("✓ Execution complete");
                println!("  Artifacts generated: {}", result.artifacts.len());
                println!("  Duration: {:?}", result.duration);
                if result.iterations > 0 {
                    println!("  Iterations: {}", result.iterations);
                }
                
                // List artifacts
                if !result.artifacts.is_empty() {
                    println!("\nGenerated Files:");
                    for artifact in &result.artifacts {
                        println!("  • {} ({} lines)", artifact.path, artifact.line_count);
                    }
                }
            }
            Err(e) => {
                eprintln!("\n✗ Execution failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_analyze(repo: &PathBuf, format: &str) {
    println!("Analyzing repository: {}", repo.display());
    
    let context = match RepoContext::from_path(repo) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let analyzed = match RepoAnalyzer::new().analyze(&context) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: Analysis failed: {}", e);
            std::process::exit(1);
        }
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&context).unwrap());
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&context).unwrap());
        }
        _ => {
            println!("\nRepository Analysis");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Files: {}", context.file_count);
            println!("  Total Lines: ~{}", analyzed.file_count() * 50); // Estimate
            println!("  Patterns detected: {}", analyzed.patterns().len());
            
            if !analyzed.patterns().is_empty() {
                println!("\nDetected Patterns:");
                for pattern in analyzed.patterns() {
                    println!("  • {:?} (confidence: {:.0}%)", pattern.pattern_type, pattern.confidence * 100.0);
                }
            }
            
            println!("\nDependencies: {}", context.dependencies.len());
            if !context.dependencies.is_empty() {
                for dep in context.dependencies.iter().take(10) {
                    println!("  • {} v{}", dep.name, dep.version);
                }
                if context.dependencies.len() > 10 {
                    println!("  ... and {} more", context.dependencies.len() - 10);
                }
            }
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_plan(package: &PathBuf, repo: &PathBuf, format: &str) {
    println!("Planning implementation...\n");
    
    let work_package = match load_work_package(package) {
        Ok(wp) => wp,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let context = match RepoContext::from_path(repo) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let analyzed = match RepoAnalyzer::new().analyze(&context) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: Analysis failed: {}", e);
            std::process::exit(1);
        }
    };

    let planner = ImplementationPlanner::new();
    let plan = match planner.plan(&work_package, &analyzed) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Planning failed: {}", e);
            std::process::exit(1);
        }
    };

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&plan).unwrap());
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&plan).unwrap());
        }
        _ => {
            println!("Implementation Plan: {}", work_package.title);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Package ID: {}", work_package.id);
            println!("Complexity: {}/10", plan.complexity.overall);
            println!("Steps: {}\n", plan.steps.len());

            for (i, step) in plan.steps.iter().enumerate() {
                let icon = match step.step_type {
                    dooz_code::types::StepType::CreateFile => "📄",
                    dooz_code::types::StepType::CreateTest => "🧪",
                    dooz_code::types::StepType::ModifyFile => "✏️",
                    dooz_code::types::StepType::DeleteFile => "🗑️",
                    dooz_code::types::StepType::Verify => "✓",
                    _ => "•",
                };
                println!("{} Step {}: {}", icon, i + 1, step.description);
                if !step.target.is_empty() {
                    println!("   Target: {}", step.target);
                }
            }

            if !plan.test_plan.test_files.is_empty() {
                println!("\nTest Plan:");
                for test in &plan.test_plan.test_files {
                    println!("  🧪 {}", test);
                }
            }

            if !plan.rollback.steps.is_empty() {
                println!("\nRollback Plan: {} steps", plan.rollback.steps.len());
            }
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_validate(package: &PathBuf) {
    println!("Validating work package: {}", package.display());
    
    match load_work_package(package) {
        Ok(wp) => {
            println!("\n✓ Work package is valid");
            println!("  ID: {}", wp.id);
            println!("  Title: {}", wp.title);
            println!("  Criteria: {}", wp.criteria.len());
            println!("  Scope Items: {}", wp.scope.items.len());
            println!("  Approved: {}", if wp.is_approved() { "Yes" } else { "No" });
            
            if !wp.is_approved() {
                println!("\n⚠️  Warning: Package is not approved and cannot be executed");
            }
        }
        Err(e) => {
            eprintln!("\n✗ Invalid work package: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_info() {
    println!("Dooz-Code v{}", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("An autonomous coder that belongs inside a company\n");
    
    println!("Capabilities:");
    println!("  ✓ Execute approved work packages");
    println!("  ✓ Analyze repository context");
    println!("  ✓ Generate implementation plans");
    println!("  ✓ Validate work packages\n");
    
    println!("Constitutional Constraints:");
    println!("  ✗ No network access");
    println!("  ✗ No autonomous decisions");
    println!("  ✗ No scope expansion");
    println!("  ✗ No unapproved execution\n");
    
    println!("For more information: https://github.com/DoozHub/dooz-code");
}

#[cfg(feature = "binary")]
fn format_step_type(step_type: &dooz_code::types::StepType) -> &'static str {
    match step_type {
        dooz_code::types::StepType::CreateFile => "CREATE",
        dooz_code::types::StepType::CreateTest => "TEST",
        dooz_code::types::StepType::ModifyFile => "MODIFY",
        dooz_code::types::StepType::AddContent => "ADD",
        dooz_code::types::StepType::ReplaceContent => "REPLACE",
        dooz_code::types::StepType::RemoveContent => "REMOVE",
        dooz_code::types::StepType::DeleteFile => "DELETE",
        dooz_code::types::StepType::UpdateConfig => "CONFIG",
        dooz_code::types::StepType::Verify => "VERIFY",
    }
}

#[cfg(feature = "binary")]
fn load_work_package(path: &PathBuf) -> Result<WorkPackage, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "json" => serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON: {}", e)),
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .map_err(|e| format!("Invalid YAML: {}", e)),
        _ => Err("Unknown file format. Use .json or .yaml".to_string()),
    }
}

#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("Binary not enabled. Build with: cargo build --features binary");
    std::process::exit(1);
}
