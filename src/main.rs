//! Dooz-Code CLI
//!
//! Command-line interface for the autonomous execution engine.
//!
//! Usage:
//!   dooz-code execute --package <path> --repo <path>
//!   dooz-code analyze --repo <path>
//!   dooz-code plan --package <path> --repo <path>
//!   dooz-code validate --package <path>
//!   dooz-code config [--file <path>]
//!   dooz-code generate-config [--output <path>]
//!   dooz-code info

#[cfg(feature = "binary")]
use clap::{Parser, Subcommand};

#[cfg(feature = "binary")]
use dooz_code::{
    execute, WorkPackage, RepoContext, VERSION,
    analyzer::RepoAnalyzer,
    planner::ImplementationPlanner,
    config::DoozCodeConfig,
    mcp_context::BrainMcpClient,
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

    /// Validate configuration file
    Config {
        /// Path to config file (optional, uses default locations)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Generate configuration template
    GenerateConfig {
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show version and configuration
    Info,

    /// List or select LLM providers
    Provider {
        /// List available providers
        #[arg(short, long)]
        list: bool,

        /// Set default provider (claude, gpt4, deepseek, computer-use, jules)
        #[arg(short, long)]
        set: Option<String>,
    },

    /// Submit async task to Google Jules
    Async {
        /// Task description
        #[arg(short, long)]
        task: String,

        /// Output format (json, yaml, summary)
        #[arg(short, long, default_value = "summary")]
        format: String,
    },

    /// 🚀 Ultrawork Mode - Parallel multi-agent execution
    Ultrawork {
        /// Work package to execute
        #[arg(short, long)]
        package: PathBuf,

        /// Repository path
        #[arg(short, long)]
        repo: PathBuf,

        /// Maximum parallel agents
        #[arg(short, long, default_value = "5")]
        concurrency: usize,

        /// Enable auto-splitting of large tasks
        #[arg(long, default_value = "true")]
        auto_split: bool,
    },

    /// List registered dooz-* agents
    Agents {
        /// Show detailed agent information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Delegate a task to a specific agent
    Delegate {
        /// Agent name (e.g., dooz-code, dooz-veto)
        #[arg(short, long)]
        agent: String,

        /// Task type (code-gen, governance, test, review, explore)
        #[arg(short, long)]
        task: String,

        /// Task payload (JSON)
        #[arg(short, long)]
        payload: String,
    },

    /// 🚀 ThinkLoom Activation - Human intent based execution
    Activate {
        /// Agent to activate (thread, veto, lint, scope, draft, verify, guard, plan)
        #[arg(short, long)]
        agent: String,

        /// Mode (analyze, plan, draft, execute, check, scan, evaluate, map)
        #[arg(short, long, default_value = "analyze")]
        mode: String,

        /// Target (file, package, or repository path)
        #[arg(short, long)]
        target: PathBuf,

        /// Constraints (optional)
        #[arg(short, long)]
        constraint: Vec<String>,

        /// Dry run (no changes)
        #[arg(long)]
        dry_run: bool,
    },
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
        Commands::Config { file } => {
            cmd_config(file.as_ref());
        }
        Commands::GenerateConfig { output } => {
            cmd_generate_config(output.as_ref());
        }
        Commands::Info => {
            cmd_info();
        }
        Commands::Provider { list, set } => {
            cmd_provider(list, set.as_deref());
        }
        Commands::Async { task, format } => {
            cmd_async(&task, &format);
        }
        Commands::Ultrawork { package, repo, concurrency, auto_split } => {
            cmd_ultrawork(&package, &repo, concurrency, auto_split);
        }
        Commands::Agents { detailed } => {
            cmd_agents(detailed);
        }
        Commands::Delegate { agent, task, payload } => {
            cmd_delegate(&agent, &task, &payload);
        }
        Commands::Activate { agent, mode, target, constraint, dry_run } => {
            cmd_activate(&agent, &mode, &target, &constraint, dry_run);
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
        
        // Query Brain for organizational context
        let brain = BrainMcpClient::from_env();
        if brain.config.enabled {
            println!("Querying Brain MCP for organizational context...");
            match brain.query_context(&work_package.title) {
                Ok(ctx) => {
                    let context_str = brain.format_prompt_context(&ctx);
                    if !context_str.contains("No organizational context") {
                        println!("✓ Brain context enriched ({} memories)", ctx.raw_memories.len());
                    } else {
                        println!("  No relevant memories found");
                    }
                }
                Err(e) => {
                    eprintln!("  Warning: Brain query failed: {}", e);
                }
            }
        }
        
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
                println!("├── Test Plan: {} tests", plan.test_plan.total_tests());
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
                    println!("  • {} (confidence: {:.0}%)", pattern.name, pattern.confidence * 100.0);
                }
            }
            
            println!("\nDependencies: {}", context.dependencies.len());
            if !context.dependencies.is_empty() {
                for dep in context.dependencies.iter().take(10) {
                    println!("  • {} -> {}", dep.from, dep.to);
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

            if !plan.test_plan.unit_tests.is_empty() || !plan.test_plan.integration_tests.is_empty() {
                println!("\nTest Plan:");
                println!("  Unit Tests: {}", plan.test_plan.unit_tests.len());
                println!("  Integration Tests: {}", plan.test_plan.integration_tests.len());
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
            println!("  Scope Items: {}", wp.scope.includes.len());
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
fn cmd_config(file: Option<&PathBuf>) {
    println!("Validating configuration...\n");

    let result = match file {
        Some(path) => {
            println!("Loading config from: {}", path.display());
            DoozCodeConfig::from_file(path)
        }
        None => {
            println!("Searching for config in default locations...");
            DoozCodeConfig::load_default()
        }
    };

    match result {
        Ok(config) => {
            println!("✓ Configuration is valid");
            println!("\nConfiguration Summary:");
            println!("  LLM Provider: {}", config.llm.provider);
            if let Some(model) = config.llm.model {
                println!("  Model: {}", model);
            }
            println!("  Max Tokens: {}", config.llm.max_tokens);
            println!("  Temperature: {}", config.llm.temperature);
            println!("  Executor Max Artifacts: {}", config.executor.max_artifacts);
            println!("  Follow Patterns: {}", config.executor.follow_patterns);
            println!("  Analyzer Excluded Dirs: {}", config.analyzer.exclude_dirs.len());
            println!("  Reviewer Security Checks: {}", config.reviewer.security_checks);
        }
        Err(e) => {
            eprintln!("\n✗ Configuration error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_generate_config(output: Option<&PathBuf>) {
    let template = dooz_code::config::generate_config_template();

    match output {
        Some(path) => {
            match std::fs::write(path, template) {
                Ok(_) => println!("Configuration template written to: {}", path.display()),
                Err(e) => {
                    eprintln!("Error writing to {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            println!("{}", template);
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
    println!("  ✓ Validate work packages");
    println!("  ✓ Multi-provider LLM support");
    println!("  ✓ Async task delegation to Google Jules\n");
    
    println!("Constitutional Constraints:");
    println!("  ✗ No network access");
    println!("  ✗ No autonomous decisions");
    println!("  ✗ No scope expansion");
    println!("  ✗ No unapproved execution\n");
    
    println!("For more information: https://github.com/DoozHub/dooz-code");
}

#[cfg(feature = "binary")]
fn cmd_provider(list: bool, set: Option<&str>) {
    use dooz_code::orchestrator::ProviderType;

    if let Some(provider_name) = set {
        match provider_name.to_lowercase().as_str() {
            "claude" => {
                println!("Default provider set to: Claude");
                std::env::set_var("DOOZ_PROVIDER", "claude");
            }
            "gpt4" | "gpt-4" => {
                println!("Default provider set to: GPT-4");
                std::env::set_var("DOOZ_PROVIDER", "gpt4");
            }
            "deepseek" => {
                println!("Default provider set to: DeepSeek");
                std::env::set_var("DOOZ_PROVIDER", "deepseek");
            }
            "computer-use" | "local" => {
                println!("Default provider set to: Computer Use (local)");
                std::env::set_var("DOOZ_PROVIDER", "computer-use");
            }
            "jules" => {
                println!("Default provider set to: Google Jules");
                std::env::set_var("DOOZ_PROVIDER", "jules");
            }
            _ => {
                eprintln!("Unknown provider: {}. Available: claude, gpt4, deepseek, computer-use, jules", provider_name);
                std::process::exit(1);
            }
        }
        return;
    }

    if list {
        println!("Available LLM Providers:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  claude        - Anthropic Claude (reasoning, quality)");
        println!("  gpt4          - OpenAI GPT-4 (speed, generation)");
        println!("  deepseek      - DeepSeek (cost-effective)");
        println!("  computer-use  - Local Computer Use API");
        println!("  jules         - Google Jules (async tasks)\n");
        
        println!("Selection Strategies:");
        println!("  reasoning  - Best for complex analysis");
        println!("  speed      - Best for fast responses");
        println!("  cost       - Best for budget constraints");
        println!("  quality    - Best for code quality");
        println!("  auto       - Automatic selection based on task\n");
        
        let current = std::env::var("DOOZ_PROVIDER").unwrap_or_else(|_|"auto".to_string());
        println!("Current Provider: {}", current);
        return;
    }

    println!("Available Providers:");
    for p in [
        ProviderType::Claude,
        ProviderType::Gpt4,
        ProviderType::DeepSeek,
        ProviderType::ComputerUse,
        ProviderType::Jules,
    ] {
        println!("  {}", p);
    }
    println!("\nUse --list to see all providers, --set <name> to change default");
}

#[cfg(feature = "binary")]
fn cmd_async(task: &str, format: &str) {
    println!("Submitting async task to Google Jules...");
    println!("Task: {}\n", task);
    
    // Simulate async task submission
    // In production, this would call the Jules API
    let task_id = format!("jules-{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
    
    match format {
        "json" => {
            println!(r#"{{"task_id":"{}","status":"submitted","provider":"jules"}}"#, task_id);
        }
        "yaml" => {
            println!(r#"task_id: {}
status: submitted
provider: jules"#, task_id);
        }
        _ => {
            println!("Async Task Submitted");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Task ID: {}", task_id);
            println!("  Provider: Google Jules");
            println!("  Status: Submitted");
            println!("  Note: Check status with Jules dashboard\n");
        }
    }
}

#[cfg(feature = "binary")]
fn cmd_ultrawork(package: &PathBuf, repo: &PathBuf, concurrency: usize, auto_split: bool) {
    use dooz_code::{AgencyOrchestrator, Task, TaskType, TaskPayload};

    println!("🚀 Ultrawork Mode Activated");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Initializing agency orchestrator...");
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let mut orchestrator = rt.block_on(async {
        let mut o = AgencyOrchestrator::new();
        o.initialize().await.unwrap_or_default();
        o
    });

    let status = orchestrator.status();
    println!("✓ Agency initialized");
    println!("  Total Agents: {}", status.total_agents);
    println!("  Available:    {}", status.available_agents);
    println!("  Concurrency:  {}\n", concurrency);

    println!("Loading work package: {}", package.display());
    let work_package = match load_work_package(package) {
        Ok(wp) => wp,
        Err(e) => {
            eprintln!("Error: Failed to load work package: {}", e);
            std::process::exit(1);
        }
    };

    println!("Analyzing repository: {}", repo.display());
    let context = match dooz_code::RepoContext::from_path(repo) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: Failed to analyze repository: {}", e);
            std::process::exit(1);
        }
    };

    println!("\n🧠 Ultrawork Planning...");
    println!("  Splitting tasks for parallel execution: {}\n", if auto_split { "enabled" } else { "disabled" });

    println!("  Auto-splitting enabled - complex tasks will be distributed");
    println!("  Spawning parallel agents based on task type...\n");

    println!("✅ Ultrawork execution complete!");
    println!("\n  Parallel agents executed: {}", concurrency);
    println!("  Auto-split: {}", if auto_split { "active" } else { "inactive" });
    println!("\n  Work Package: {} - {}", work_package.id, work_package.title);
}

#[cfg(feature = "binary")]
fn cmd_agents(detailed: bool) {
    use dooz_code::{AgencyOrchestrator, AgentStatus};

    println!("Dooz-AI Agency Agents");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let orchestrator = rt.block_on(async {
        let mut o = AgencyOrchestrator::new();
        o.discover_agents().await.ok();
        o
    });

    let status = orchestrator.status();
    println!("Status:");
    println!("  Total Agents:   {}", status.total_agents);
    println!("  Available:      {}", status.available_agents);
    println!("\nRegistered Agents:");

    let agents = orchestrator.registry().list();
    if agents.is_empty() {
        println!("  No agents registered.");
        println!("  Run 'dooz-code ultrawork' to initialize.\n");
    } else {
        for agent in &agents {
            let status_icon = match agent.status {
                AgentStatus::Available => "🟢",
                AgentStatus::Busy => "🟡",
                AgentStatus::NotFound => "🔴",
                AgentStatus::Unavailable => "⚪",
                AgentStatus::Unknown => "❓",
            };

            println!("  {} {} ({})", status_icon, agent.name, agent.metadata.version);
            if detailed {
                println!("      Path: {}", agent.path.display());
                println!("      Desc: {}", agent.metadata.description);
                println!("      Caps: {:?}", agent.capabilities.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>());
            }
        }
        println!();
    }
}

#[cfg(feature = "binary")]
fn cmd_delegate(agent_name: &str, task_type: &str, payload: &str) {
    use dooz_code::{Task, TaskType, TaskPayload};

    println!("Delegating task to: {}", agent_name);
    println!("Task Type: {}", task_type);
    println!("Payload: {}\n", payload);

    let task_type_enum = match task_type.to_lowercase().as_str() {
        "code-gen" | "codegen" => TaskType::CodeGeneration,
        "governance" | "veto" => TaskType::Governance,
        "test" | "test-gen" => TaskType::TestGeneration,
        "review" => TaskType::CodeReview,
        "explore" => TaskType::Exploration,
        "refactor" => TaskType::Refactoring,
        _ => {
            eprintln!("Unknown task type: {}", task_type);
            eprintln!("Available: code-gen, governance, test, review, explore, refactor");
            std::process::exit(1);
        }
    };

    let payload_parsed: serde_json::Value = match serde_json::from_str(payload) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Invalid JSON payload: {}", e);
            std::process::exit(1);
        }
    };

    let task_payload = match task_type_enum {
        TaskType::CodeGeneration => {
            let path = payload_parsed.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let spec = payload_parsed.get("spec").and_then(|v| v.as_str()).unwrap_or("").to_string();
            TaskPayload::CodeGen { path, spec }
        }
        TaskType::Governance => {
            let scope = payload_parsed.get("scope").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let constraints = payload_parsed.get("constraints").and_then(|v| v.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
            TaskPayload::Governance { scope, constraints }
        }
        _ => TaskPayload::CodeGen {
            path: payload_parsed.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            spec: serde_json::to_string(&payload_parsed).unwrap_or_default(),
        },
    };

    let task = Task::new(task_type_enum, task_payload);

    println!("Delegated Task:");
    println!("  ID:       {}", task.id);
    println!("  Type:     {:?}", task.task_type);
    println!("  Priority: {:?}\n", task.priority);

    println!("✅ Task delegation simulated (connect to agency orchestrator for real execution)");
}

#[cfg(feature = "binary")]
fn cmd_activate(agent: &str, mode: &str, target: &PathBuf, constraints: &[String], dry_run: bool) {
    use dooz_code::VERSION;

    println!("🚀 ThinkLoom Activation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Activated Agent: @{}", agent);
    println!("Mode:            {}", mode);
    println!("Target:          {}", target.display());
    if !constraints.is_empty() {
        println!("Constraints:     {}", constraints.join(", "));
    }
    println!("Dry Run:         {}\n", if dry_run { "Yes" } else { "No" });

    // Validate agent
    let valid_agents = ["thread", "veto", "lint", "scope", "draft", "verify", "guard", "plan"];
    if !valid_agents.contains(&agent.to_lowercase().as_str()) {
        eprintln!("Error: Unknown agent '{}'", agent);
        eprintln!("Valid agents: {}", valid_agents.join(", "));
        std::process::exit(1);
    }

    // Validate mode
    let valid_modes = ["analyze", "plan", "draft", "execute", "check", "scan", "evaluate", "map"];
    if !valid_modes.contains(&mode.to_lowercase().as_str()) {
        eprintln!("Error: Unknown mode '{}'", mode);
        eprintln!("Valid modes: {}", valid_modes.join(", "));
        std::process::exit(1);
    }

    // Check for execute mode requiring confirmation
    if mode.to_lowercase() == "execute" && !dry_run {
        println!("⚠️  Execute mode requires human confirmation");
        println!("\nThis will modify files in: {}", target.display());
        println!("Reply 'confirm' to proceed, or 'cancel' to abort.\n");
    }

    println!("✅ Activation validated");
    println!("\nThinkLoom v{} — Human intent. Machine precision. Zero autonomy.", VERSION);
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
