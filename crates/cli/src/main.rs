use clap::{Parser, Subcommand};
use dialoguer::{Input, MultiSelect};
use dijiang_configurator::PlatformKind;
use dijiang_configurator::TemplateRegistry;
use dijiang_task::store;
use dijiang_task::types::{TaskRecord, TaskStatus};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};

mod commands;
mod util;
#[derive(Parser)]
#[command(
    name = "dijiang",
    version = env!("CARGO_PKG_VERSION"),
    about = "DiJiang - AI coding assistant workflow layer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示项目状态
    Status {
        /// 显示详细的兼容性诊断信息
        #[arg(long)]
        compat: bool,
    },
    /// 开始并激活一个 DiJiang 工作会话（生命周期入口）
    Start {
        /// 任务名称（slug 格式，如 "fix-login-bug"）
        name: String,
        /// 任务显示标题（可选）
        title: Option<String>,
    },
    /// 接收自然语言请求，自动创建/激活任务并输出路由上下文
    Dispatch {
        /// 用户原始请求
        prompt: Vec<String>,
        /// 已有 active task 时仍创建新任务
        #[arg(long)]
        force_new: bool,
        /// 输出 Codex/Agent hook 可消费的 JSON payload
        #[arg(long)]
        json: bool,
        /// hook event name（JSON 输出时使用）
        #[arg(long, default_value = "UserPromptSubmit")]
        hook_event: String,
    },
    /// 底层任务管理（原子状态操作）
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// 初始化 DiJiang 项目
    Init {
        /// 项目名称（默认为目录名）
        #[arg(default_value = "")]
        name: String,
        /// 开发者名称（如未提供则从 git 检测）
        #[arg(long)]
        developer: Option<String>,
        /// 跳过交互式提示，使用默认值
        #[arg(long, short = 'y')]
        yes: bool,
        /// 强制重新初始化
        #[arg(long)]
        force: bool,
        /// 要配置的平台（逗号分隔：pi,cursor,claude,codex,opencode,hermes）
        #[arg(long)]
        platforms: Option<String>,
        /// 自动检测已安装的平台
        #[arg(long)]
        auto_detect: bool,
    },
    /// 记忆管理：项目 findings/lessons 与全局 tactics/patterns
    Mem {
        #[command(subcommand)]
        command: MemCommands,
    },
    /// 模板管理
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// 管理 dj-* 技能（列出、同步到项目）
    Skills {
        /// 同步技能到当前项目
        #[arg(long)]
        sync: bool,
    },
    /// 输出当前 session 的 workflow state（供 hook/agent 注入使用）
    WorkflowState {
        /// 输出 Codex/Agent hook 可消费的 JSON payload
        #[arg(long)]
        json: bool,
        /// hook event name（JSON 输出时使用）
        #[arg(long, default_value = "UserPromptSubmit")]
        hook_event: String,
    },
    /// 按 skill 名称读取共享 body registry 中的目标 skill 正文（执行期 lazy fetch 通路）
    SkillBody {
        /// skill name, e.g. dj-tdd
        name: String,
        /// 输出 JSON payload
        #[arg(long)]
        json: bool,
    },
    /// 将 Trellis 项目迁移到 DiJiang
    Migrate,
    /// Agent 通道管理
    Channel {
        #[command(subcommand)]
        command: ChannelCommands,
    },
    /// 完成当前工作：验证、文档同步门禁、可选提交/集成、归档任务、记录 journal
    FinishWork {
        /// 本次工作的简短总结
        #[arg(long)]
        summary: Option<String>,
        /// 已执行的验证命令或人工检查结论
        #[arg(long)]
        verification: Option<String>,
        /// 文档/spec 同步结论；有代码变更或 --commit 时必填
        #[arg(long)]
        docs_sync: Option<String>,
        /// 版本影响决策：major、minor、patch、none
        #[arg(long, default_value = "none")]
        version_impact: String,
        /// 提交当前任务 diff 后再归档
        #[arg(long)]
        commit: bool,
        /// 提交消息；未提供时根据 task 和 summary 自动生成
        #[arg(long)]
        commit_message: Option<String>,
        /// push 任务分支；只在 --commit 后允许
        #[arg(long)]
        push: bool,
        /// 合并任务分支到主分支并清理 worktree；只在 --commit 后允许
        #[arg(long)]
        integrate: bool,
        /// 显式批准高风险的 finish-work integration / push
        #[arg(long)]
        approve_integrate: bool,
        /// 显式批准高风险的 finish-work cleanup（worktree remove / branch delete）
        #[arg(long)]
        approve_cleanup: bool,
        /// 集成目标主分支
        #[arg(long, default_value = "main")]
        main_branch: String,
        /// 远端名称
        #[arg(long, default_value = "origin")]
        remote: String,
        /// 允许在 git 工作区存在未提交/未跟踪改动时不提交也完成任务
        #[arg(long)]
        allow_dirty: bool,
        /// 保留任务 worktree（默认自动清理）
        #[arg(long)]
        keep_worktree: bool,
    },
    /// 同步 spec 文件 checksums：检查是否有 specs 发生变化
    SpecSync {
        #[command(subcommand)]
        command: SpecSyncCommands,
    },
    /// 更新当前项目的 dj-* 技能和代理
    Update {
        /// 强制更新所有文件
        #[arg(long)]
        force: bool,
        /// 从 GitHub 下载最新版本
        #[arg(long)]
        from_github: bool,
    },
    DocSync {
        #[command(subcommand)]
        command: DocSyncCommands,
    },
}

#[derive(Subcommand)]
enum DocSyncCommands {
    /// 扫描 git diff，输出受代码变更影响的文档清单（不改文件）
    Check {
        /// 对比的基础分支（默认 main）
        #[arg(long, default_value = "main")]
        base: String,
    },
}

#[derive(Subcommand)]
enum SpecSyncCommands {
    /// 检查当前 spec 文件是否与已记录的 checksums 不同
    Check,
    /// 记录当前 spec 文件 checksums 到 `.dijiang/.runtime/`
    Record,
}

#[derive(Subcommand)]
enum ChannelCommands {
    /// 生成一个 agent 执行任务
    Spawn {
        /// Agent 名称（check, implement 等）
        agent: String,
        /// 任务路径（可选，默认为当前任务）
        #[arg(long)]
        task: Option<String>,
        /// 工作目录（可选）
        #[arg(long)]
        dir: Option<String>,
    },
    /// 列出活跃的通道
    List,
    /// 向通道发送消息
    Send {
        /// 通道 ID
        channel_id: String,
        /// 要发送的消息
        message: String,
    },
    /// 查看通道状态
    Status {
        /// 通道 ID（或 'all' 查看所有通道）
        channel_id: String,
    },
    /// 停止通道
    Stop {
        /// 通道 ID
        channel_id: String,
    },
    /// 在通道中执行 agent
    Execute {
        /// 通道 ID
        channel_id: String,
        /// 使用的模型（可选）
        #[arg(long)]
        model: Option<String>,
        /// 使用的提供商（可选）
        #[arg(long)]
        provider: Option<String>,
        /// 超时时间（秒，默认 300）
        #[arg(short, long, default_value = "300")]
        timeout: u64,
        /// 实时输出
        #[arg(long)]
        follow: bool,
    },
    /// 并行执行所有活跃的通道
    ExecuteAll {
        /// 使用的模型（可选）
        #[arg(long)]
        model: Option<String>,
        /// 使用的提供商（可选）
        #[arg(long)]
        provider: Option<String>,
        /// 超时时间（秒，默认 300）
        #[arg(short, long, default_value = "300")]
        timeout: u64,
    },
}
#[derive(Subcommand)]
enum MemCommands {
    /// 列出跨平台会话
    List,
    /// 同步所有平台会话到 ~/.dijiang/mem/
    Sync,
    /// 追加发现到项目日志
    Findings {
        #[arg(long)]
        finding: String,
    },
    /// 记录学习到项目日志
    Learn {
        #[arg(long)]
        lesson: String,
    },
    /// 记录用户纠正并吸收为经验教训
    Correction {
        #[arg(long)]
        correction: String,
        #[arg(long)]
        lesson: String,
        #[arg(long, default_value = "workflow")]
        scope: String,
        #[arg(long, default_value = "user")]
        source: String,
        #[arg(long, default_value = "until-superseded")]
        freshness: String,
        #[arg(long, default_value = "none")]
        conflict: String,
        #[arg(long)]
        actionability: String,
    },
    /// 归档当前会话
    Archive,
    /// 添加策略到全局记忆
    Tactic {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
    },
    /// 列出策略或通过 Thompson 采样选择 top-k
    Tactics {
        #[arg(long, default_value = "5")]
        select: usize,
    },
    /// 记录策略到实验记录并更新策略统计
    Record {
        #[arg(long)]
        tactic: String,
        #[arg(long)]
        outcome: String,
        #[arg(long)]
        context: String,
    },
    /// 根据关键词召回项目记忆（findings/learnings/patterns）
    Recall {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "5")]
        limit: usize,
        #[arg(long)]
        project: Option<String>,
    },
    /// 重建倒排索引（FTS 替代方案，为 recall 加速）
    Index,
    /// 清理 N 天之前的过期记忆
    Prune {
        #[arg(long, default_value = "90")]
        days: u64,
    },
    /// 添加模式/标准操作流程
    Pattern {
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: String,
    },
    /// 列出模式
    Patterns,
    /// 显示记忆统计信息
    Stats,
    /// 备份项目记忆到全局
    Backup,
    /// 运行快速进化循环（分析会话，提炼策略）
    Evolve,
    /// 运行慢速微调循环（基于积累的经验训练）
    Finetune,
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// 列出可用模板（内置和缓存）
    List,
    /// 从源拉取模板（gh:owner/repo 或 URL）
    Pull {
        /// 模板源（如 gh:tiezhu/dijiang-templates）
        source: String,
    },
    /// 验证模板目录
    Validate {
        /// 模板目录或 manifest.toml 路径
        path: String,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List all tasks
    List,
    /// Show current active task
    Current,
    /// Create and activate a task record (low-level task operation)
    Start {
        /// Task name (slug, e.g. "fix-login-bug")
        name: String,
    },
    /// Set task status
    Status {
        /// Task name (slug)
        name: String,
        /// New status: planning|in_progress|completed|archived|paused
        status: String,
    },
    /// Archive a task (set status to Archived, record archived_at)
    Archive {
        /// Task name (slug)
        name: String,
    },
    /// Prune old archived tasks
    Prune {
        /// Prune tasks archived more than N days ago (default: 30)
        #[arg(long, default_value = "30")]
        days: u64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name, title } => commands::start::cmd_start(&name, title.as_deref()),
        Commands::Dispatch {
            prompt,
            force_new,
            json,
            hook_event,
        } => commands::dispatch::cmd_dispatch(&prompt.join(" "), force_new, json, &hook_event),
        Commands::Status { compat } => commands::status::cmd_status(compat),
        Commands::Init {
            name,
            developer,
            yes,
            force,
            platforms,
            auto_detect,
        } => commands::init::cmd_init(
            &name,
            developer.as_deref(),
            yes,
            force,
            platforms.as_deref(),
            auto_detect,
        ),
        Commands::Task { command } => match command {
            TaskCommands::List => commands::task::cmd_task_list(),
            TaskCommands::Current => commands::task::cmd_task_current(),
            TaskCommands::Start { name } => commands::task::cmd_task_start(&name),
            TaskCommands::Status { name, status } => commands::task::cmd_task_status(&name, &status),
            TaskCommands::Archive { name } => commands::task::cmd_task_archive(&name),
            TaskCommands::Prune { days } => commands::task::cmd_task_prune(days),
        },
        Commands::Mem {
            command: MemCommands::List,
        } => commands::mem::cmd_mem_list(),
        Commands::Mem {
            command: MemCommands::Sync,
        } => commands::mem::cmd_mem_sync(),
        Commands::Mem {
            command: MemCommands::Findings { finding },
        } => commands::mem::cmd_mem_findings(&finding),
        Commands::Mem {
            command: MemCommands::Learn { lesson },
        } => commands::mem::cmd_mem_learn(&lesson),
        Commands::Mem {
            command:
                MemCommands::Correction {
                    correction,
                    lesson,
                    scope,
                    source,
                    freshness,
                    conflict,
                    actionability,
                },
        } => commands::mem::cmd_mem_correction(
            &correction,
            &lesson,
            &scope,
            &source,
            &freshness,
            &conflict,
            &actionability,
        ),
        Commands::Mem {
            command: MemCommands::Archive,
        } => commands::mem::cmd_mem_archive(),
        Commands::Mem {
            command: MemCommands::Tactic { name, description },
        } => commands::mem::cmd_mem_tactic(&name, &description),
        Commands::Mem {
            command: MemCommands::Tactics { select },
        } => commands::mem::cmd_mem_tactics(select),
        Commands::Mem {
            command:
                MemCommands::Record {
                    tactic,
                    outcome,
                    context,
                },
        } => commands::mem::cmd_mem_record(&tactic, &outcome, &context),
        Commands::Mem {
            command: MemCommands::Pattern { name, description },
        } => commands::mem::cmd_mem_pattern(&name, &description),
        Commands::Mem {
            command: MemCommands::Patterns,
        } => commands::mem::cmd_mem_patterns(),
        Commands::Mem {
            command: MemCommands::Stats,
        } => commands::mem::cmd_mem_stats(),
        Commands::Mem {
            command: MemCommands::Backup,
        } => commands::mem::cmd_mem_backup(),
        Commands::Mem {
            command: MemCommands::Evolve,
        } => commands::mem::cmd_mem_evolve(),
        Commands::Mem {
            command: MemCommands::Finetune,
        } => commands::mem::cmd_mem_finetune(),
        Commands::Mem {
            command: MemCommands::Recall { query, limit, project },
        } => commands::mem::cmd_mem_recall(&query, limit, project.as_deref()),
        Commands::Mem {
            command: MemCommands::Index,
        } => commands::mem::cmd_mem_index(),
        Commands::Mem {
            command: MemCommands::Prune { days },
        } => commands::mem::cmd_mem_prune(days),
        Commands::Template { command } => match command {
            TemplateCommands::List => commands::template::cmd_template_list(),
            TemplateCommands::Pull { source } => commands::template::cmd_template_pull(&source),
            TemplateCommands::Validate { path } => commands::template::cmd_template_validate(&path),
        },
        Commands::Skills { sync } => commands::skills::cmd_skills(sync),
        Commands::Migrate => commands::migrate::cmd_migrate(),
        Commands::WorkflowState { json, hook_event } => commands::workflow::cmd_workflow_state(json, &hook_event),
        Commands::SkillBody { name, json } => commands::workflow::cmd_skill_body(&name, json),
        Commands::Channel { command } => match command {
            ChannelCommands::Spawn { agent, task, dir } => {
                commands::channel::cmd_channel_spawn(&agent, task.as_deref(), dir.as_deref())
            }
            ChannelCommands::List => commands::channel::cmd_channel_list(),
            ChannelCommands::Send {
                channel_id,
                message,
            } => commands::channel::cmd_channel_send(&channel_id, &message),
            ChannelCommands::Status { channel_id } => commands::channel::cmd_channel_status(&channel_id),
            ChannelCommands::Stop { channel_id } => commands::channel::cmd_channel_stop(&channel_id),
            ChannelCommands::Execute {
                channel_id,
                model,
                provider,
                timeout,
                follow,
            } => commands::channel::cmd_channel_execute(
                &channel_id,
                model.as_deref(),
                provider.as_deref(),
                timeout,
                follow,
            ),
            ChannelCommands::ExecuteAll {
                model,
                provider,
                timeout,
            } => commands::channel::cmd_channel_execute_all(model.as_deref(), provider.as_deref(), timeout),
        },
        Commands::FinishWork {
            summary,
            verification,
            docs_sync,
            version_impact,
            commit,
            commit_message,
            push,
            integrate,
            approve_integrate,
            approve_cleanup,
            main_branch,
            remote,
            allow_dirty,
            keep_worktree,
        } => commands::finish::cmd_finish_work(commands::finish::FinishWorkOptions {
            summary: summary.as_deref(),
            verification: verification.as_deref(),
            docs_sync: docs_sync.as_deref(),
            version_impact: &version_impact,
            commit,
            commit_message: commit_message.as_deref(),
            push,
            integrate,
            approve_integrate,
            approve_cleanup,
            main_branch: &main_branch,
            remote: &remote,
            allow_dirty,
            keep_worktree,
        }),
        Commands::DocSync { command } => match command {
            DocSyncCommands::Check { base } => commands::doc_sync::cmd_doc_sync_check(base),
        },
        Commands::SpecSync { command } => match command {
            SpecSyncCommands::Check => commands::spec_sync::cmd_spec_sync_check(),
            SpecSyncCommands::Record => commands::spec_sync::cmd_spec_sync_record(),
        },
        Commands::Update { force, from_github } => commands::update::cmd_update(force, from_github),
    }
}
#[cfg(test)]
mod tests {
    use crate::commands::dispatch::{
        apply_route_gate, dispatch_context, dispatch_route, dispatch_route_for_active_task,
        dispatch_runtime_skill_context,
    };
    use dijiang_task::store;
    use dijiang_task::types::TaskStatus;

    fn status_format(status: TaskStatus) -> (String, String) {
        (
            status.as_str().to_string(),
            status.to_trellis_status().to_string(),
        )
    }

    #[test]
    fn test_status_format_planning() {
        let (s, p) = status_format(TaskStatus::Planning);
        assert_eq!(p, "plan");
        assert_eq!(s, "planning");
    }

    #[test]
    fn test_status_format_in_progress() {
        let (s, p) = status_format(TaskStatus::InProgress);
        assert_eq!(p, "implement");
        assert_eq!(s, "in_progress");
    }

    #[test]
    fn test_status_format_completed() {
        let (s, p) = status_format(TaskStatus::Completed);
        assert_eq!(p, "complete");
        assert_eq!(s, "completed");
    }

    #[test]
    fn test_status_format_paused() {
        let (s, p) = status_format(TaskStatus::Paused);
        assert_eq!(p, "in_progress");
        assert_eq!(s, "paused");
    }

    #[test]
    fn test_status_format_archived() {
        let (s, p) = status_format(TaskStatus::Archived);
        assert_eq!(p, "complete");
        assert_eq!(s, "archived");
    }

    #[test]
    fn test_dispatch_ignores_embedded_skill_context_for_visible_prompt() {
        let route = dispatch_route(
            r#"<skill name="dijiang-start">排查调试 debug bug</skill>新问题，grill未触发"#,
        );

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_vague_bug_request_to_grill() {
        let route = dispatch_route("有个 bug 帮我修一下");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_specific_bug_request_to_hunt() {
        let route = dispatch_route("登录接口报错，帮我排查并修复");

        assert_eq!(route.skill, "dj-hunt");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_vague_feature_request_to_grill() {
        let route = dispatch_route("做个导出功能");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_specific_feature_request_to_implement() {
        let route = dispatch_route("新增一个导出按钮");

        assert_eq!(route.skill, "dj-implement");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_specific_interface_request_to_implement() {
        let route = dispatch_route("新增一个导出接口");

        assert_eq!(route.skill, "dj-implement");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_dispatch_routes_vague_login_optimization_to_grill() {
        let route = dispatch_route("优化登录体验");

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_dispatch_routes_test_work_to_tdd() {
        let route = dispatch_route("补测试");

        assert_eq!(route.skill, "dj-tdd");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_session_start_dispatch_routes_planning_task_to_grill() {
        let mut task = store::create_task("needs-alignment", "Needs Alignment");
        task.status = TaskStatus::Planning;
        task.meta = serde_json::json!({
            "dispatch": { "skill": "dj-hunt" }
        });

        let route = dispatch_route_for_active_task(&task);

        assert_eq!(route.skill, "dj-grill");
        assert_eq!(route.status, TaskStatus::Planning);
    }

    #[test]
    fn test_session_start_dispatch_keeps_in_progress_hunt_route() {
        let mut task = store::create_task("bug", "Bug");
        task.status = TaskStatus::InProgress;
        task.meta = serde_json::json!({
            "dispatch": { "skill": "dj-hunt" }
        });

        let route = dispatch_route_for_active_task(&task);

        assert_eq!(route.skill, "dj-hunt");
        assert_eq!(route.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_route_gate_redirects_planning_implement_to_grill() {
        let route = dispatch_route("新增一个导出按钮");
        let dispatch = apply_route_gate(&TaskStatus::Planning, route, Some("新增一个导出按钮"));

        assert_eq!(dispatch.route.skill, "dj-grill");
        assert_eq!(dispatch.decision.action.as_str(), "redirect");
    }

    #[test]
    fn test_route_gate_routes_paused_task_to_continue() {
        let route = dispatch_route("新增一个导出按钮");
        let dispatch = apply_route_gate(&TaskStatus::Paused, route, Some("新增一个导出按钮"));

        assert_eq!(dispatch.route.skill, "dijiang-continue");
        assert_eq!(dispatch.decision.action.as_str(), "redirect");
    }

    #[test]
    fn test_route_gate_blocks_archived_task_until_restart() {
        let route = dispatch_route("新增一个导出按钮");
        let dispatch = apply_route_gate(&TaskStatus::Archived, route, Some("新增一个导出按钮"));

        assert_eq!(dispatch.route.skill, "dijiang-start");
        assert_eq!(dispatch.decision.action.as_str(), "block");
    }

    #[test]
    fn test_dispatch_runtime_skill_context_exposes_manifests_and_target_body() {
        let route = dispatch_route("补测试");
        let dispatch = apply_route_gate(&TaskStatus::InProgress, route, Some("补测试"));

        let context = dispatch_runtime_skill_context(&dispatch);

        assert!(context.contains("<dijiang-skill-manifests>"));
        assert!(context.contains("dj-implement | 功能实现与局部代码变更"));
        assert!(context.contains(&format!(
            "<dijiang-target-skill role=\"primary\" name=\"{}\">",
            dispatch.route.skill
        )));
        assert!(context.contains("summary: "));
        if dispatch.route.recommended_path.contains("-> dj-check") {
            assert!(context.contains("<dijiang-target-skill role=\"next\" name=\"dj-check\">"));
        }
    }



    #[test]
    fn test_dispatch_context_keeps_header_and_adds_target_skill_body() {
        let route = dispatch_route("新增一个导出按钮");
        let dispatch = apply_route_gate(&TaskStatus::Planning, route, Some("新增一个导出按钮"));

        let context = dispatch_context(
            "task-1",
            "Task 1",
            &dispatch,
            "<dijiang-workflow-state>state</dijiang-workflow-state>",
            None,
        );

        assert!(context.contains("<dijiang-dispatch>"));
        assert!(context.contains("action：redirect"));
        assert!(context.contains("路线：dj-grill"));
        assert!(context.contains("<dijiang-skill-manifests>"));
        assert!(context.contains("dj-grill | 需求对齐、范围澄清、问题收敛"));
        assert!(context.contains("<dijiang-target-skill role=\"primary\" name=\"dj-grill\">") );
        assert!(context.contains("<dijiang-workflow-state>state</dijiang-workflow-state>"));
    }
}
