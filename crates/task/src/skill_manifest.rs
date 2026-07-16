use std::collections::BTreeMap;

use crate::route_gate::WorkflowCapsule;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillManifestEntry {
    pub name: &'static str,
    pub summary: &'static str,
    pub phases: &'static [&'static str],
    pub risk: &'static str,
    #[serde(skip_serializing)]
    pub body: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedSkillBody {
    pub role: &'static str,
    pub name: &'static str,
    pub summary: &'static str,
}

#[derive(Debug, Default, Clone)]
pub struct SkillBodyCache {
    bodies: BTreeMap<&'static str, &'static str>,
}

const SKILL_MANIFESTS: &[SkillManifestEntry] = &[
    SkillManifestEntry {
        name: "dj-grill",
        summary: "需求对齐、范围澄清、问题收敛",
        phases: &["align"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dj-grill/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-output",
        summary: "产出或同步 PRD、design、implement 等任务文档",
        phases: &["align", "implement"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dj-output/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-implement",
        summary: "功能实现与局部代码变更",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-implement/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-tdd",
        summary: "测试驱动实现与行为回归保护",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-tdd/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-hunt",
        summary: "bug、回归和根因排查",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-hunt/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-script",
        summary: "脚本或工具实现",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-script/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-design",
        summary: "UI/UX 主导的设计实现",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-design/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-absorb",
        summary: "有选择地从外部目标中吸收融合设计模式、交互或视觉元素到自有项目中",
        phases: &["implement"],
        risk: "medium",
        body: include_str!("../../configurator/templates/skills/dj-absorb/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-check",
        summary: "质量门禁、验证 diff、回归审查",
        phases: &["check", "finish"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dj-check/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-reason",
        summary: "推理增强、系统透镜和复杂判断校准",
        phases: &["align", "implement", "check", "finish"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dj-reason/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dj-review",
        summary: "轻量只读审查",
        phases: &["check"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dj-review/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dijiang-continue",
        summary: "恢复 paused task 上下文并重新进入 workflow",
        phases: &["resume"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dijiang-continue/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dijiang-start",
        summary: "重新激活 archived task 或启动新任务",
        phases: &["idle"],
        risk: "low",
        body: include_str!("../../configurator/templates/skills/dijiang-start/SKILL.md"),
    },
    SkillManifestEntry {
        name: "dijiang-finish-work",
        summary: "执行收尾、验证汇总、归档与提交前检查",
        phases: &["finish"],
        risk: "high",
        body: include_str!("../../configurator/templates/skills/dijiang-finish-work/SKILL.md"),
    },
];

pub fn manifests_for_capsule(capsule: WorkflowCapsule) -> Vec<SkillManifestEntry> {
    let phase = capsule.as_str();
    SKILL_MANIFESTS
        .iter()
        .filter(|entry| entry.phases.iter().any(|candidate| candidate == &phase))
        .cloned()
        .collect()
}

pub fn manifest_by_name(name: &str) -> Option<SkillManifestEntry> {
    SKILL_MANIFESTS
        .iter()
        .find(|entry| entry.name == name)
        .cloned()
}

pub fn skill_body_by_name(name: &str) -> Option<&'static str> {
    manifest_by_name(name).map(|entry| entry.body)
}

pub fn select_skill_bodies(
    capsule: WorkflowCapsule,
    primary_skill: &str,
    recommended_path: &str,
) -> Vec<SelectedSkillBody> {
    let mut selected = Vec::new();

    let Some(primary) = manifest_by_name(primary_skill) else {
        return selected;
    };

    selected.push(SelectedSkillBody {
        role: "primary",
        name: primary.name,
        summary: primary.summary,
    });

    if primary.risk == "high" || matches!(capsule, WorkflowCapsule::Resume | WorkflowCapsule::Idle)
    {
        return selected;
    }

    let Some((_, tail)) = recommended_path.split_once("->") else {
        return selected;
    };

    let candidate = tail.trim();
    if candidate.is_empty() {
        return selected;
    }

    if candidate.contains('/') {
        if !matches!(capsule, WorkflowCapsule::Align | WorkflowCapsule::Implement) {
            return selected;
        }
        for branch in candidate
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if branch == primary_skill {
                continue;
            }
            if let Some(next) = manifest_by_name(branch) {
                selected.push(SelectedSkillBody {
                    role: "branch",
                    name: next.name,
                    summary: next.summary,
                });
            }
        }
        selected.truncate(3);
        return selected;
    }

    if !matches!(
        capsule,
        WorkflowCapsule::Implement | WorkflowCapsule::Check | WorkflowCapsule::Finish
    ) {
        return selected;
    }

    for (index, skill) in candidate
        .split("->")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|segment| segment.split_whitespace().next())
        .enumerate()
    {
        if skill == primary_skill {
            continue;
        }
        let role = match index {
            0 => "next",
            _ => "followup",
        };
        if let Some(next) = manifest_by_name(skill) {
            selected.push(SelectedSkillBody {
                role,
                name: next.name,
                summary: next.summary,
            });
        }
        if selected.len() >= 3 {
            break;
        }
    }

    selected
}

pub fn render_selected_skill_bodies(
    selected: &[SelectedSkillBody],
    cache: &mut SkillBodyCache,
) -> String {
    if selected.is_empty() {
        return String::new();
    }

    selected
        .iter()
        .map(|selected| {
            let body = cache.body(selected.name).unwrap_or("# Missing Skill Body\n");
            format!(
                "<dijiang-target-skill role=\"{}\" name=\"{}\">\nsummary: {}\n\n{}\n</dijiang-target-skill>",
                selected.role, selected.name, selected.summary, body
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl SkillBodyCache {
    pub fn body(&mut self, name: &str) -> Option<&'static str> {
        if let Some(body) = self.bodies.get(name) {
            return Some(*body);
        }
        let body = skill_body_by_name(name)?;
        self.bodies.insert(manifest_by_name(name)?.name, body);
        Some(body)
    }

    pub fn len(&self) -> usize {
        self.bodies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_lookup_returns_known_skill() {
        let manifest = manifest_by_name("dj-grill").expect("manifest");
        assert_eq!(manifest.name, "dj-grill");
        assert_eq!(manifest.risk, "low");
        assert!(manifest.phases.contains(&"align"));

        let reason = manifest_by_name("dj-reason").expect("reason manifest");
        assert_eq!(reason.name, "dj-reason");
        assert_eq!(reason.risk, "low");
        assert!(reason.phases.contains(&"align"));
    }

    #[test]
    fn skill_body_lookup_returns_embedded_body() {
        let body = skill_body_by_name("dj-tdd").expect("body");
        assert!(body.contains("# TDD") || body.contains("TDD"));

        let reason = skill_body_by_name("dj-reason").expect("reason body");
        assert!(reason.contains("# Reason"));
        assert!(reason.contains("系统透镜"));
    }

    #[test]
    fn manifests_for_capsule_filters_by_phase() {
        let manifests = manifests_for_capsule(WorkflowCapsule::Align);
        assert!(manifests.iter().any(|entry| entry.name == "dj-grill"));
        assert!(manifests.iter().any(|entry| entry.name == "dj-output"));
        assert!(manifests.iter().any(|entry| entry.name == "dj-reason"));
        assert!(!manifests.iter().any(|entry| entry.name == "dj-check"));
    }

    #[test]
    fn select_skill_bodies_adds_next_skill_for_linear_path() {
        let selected =
            select_skill_bodies(WorkflowCapsule::Implement, "dj-tdd", "dj-tdd -> dj-check");
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].role, "primary");
        assert_eq!(selected[0].name, "dj-tdd");
        assert_eq!(selected[1].role, "next");
        assert_eq!(selected[1].name, "dj-check");
    }

    #[test]
    fn select_skill_bodies_adds_followup_for_short_chain() {
        let selected = select_skill_bodies(
            WorkflowCapsule::Implement,
            "dj-hunt",
            "dj-hunt -> dj-implement -> dj-check",
        );
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[1].role, "next");
        assert_eq!(selected[1].name, "dj-implement");
        assert_eq!(selected[2].role, "followup");
        assert_eq!(selected[2].name, "dj-check");
    }

    #[test]
    fn select_skill_bodies_adds_branch_previews_for_branching_path() {
        let selected = select_skill_bodies(
            WorkflowCapsule::Align,
            "dj-grill",
            "dj-grill -> dj-output/dj-implement",
        );
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].name, "dj-grill");
        assert_eq!(selected[1].role, "branch");
        assert_eq!(selected[1].name, "dj-output");
        assert_eq!(selected[2].role, "branch");
        assert_eq!(selected[2].name, "dj-implement");
    }

    #[test]
    fn select_skill_bodies_keeps_resume_capsule_primary_only() {
        let selected = select_skill_bodies(
            WorkflowCapsule::Resume,
            "dijiang-continue",
            "dijiang-continue -> dj-check",
        );
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "dijiang-continue");
    }

    #[test]
    fn select_skill_bodies_blocks_followup_for_high_risk_primary() {
        let selected = select_skill_bodies(
            WorkflowCapsule::Finish,
            "dijiang-finish-work",
            "dijiang-finish-work -> dj-check",
        );
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "dijiang-finish-work");
    }

    #[test]
    fn render_selected_skill_bodies_uses_lazy_cache() {
        let selected =
            select_skill_bodies(WorkflowCapsule::Implement, "dj-tdd", "dj-tdd -> dj-check");
        let mut cache = SkillBodyCache::default();
        let rendered = render_selected_skill_bodies(&selected, &mut cache);
        assert!(rendered.contains("role=\"primary\" name=\"dj-tdd\""));
        assert!(rendered.contains("role=\"next\" name=\"dj-check\""));
        assert_eq!(cache.len(), 2);
        let _ = render_selected_skill_bodies(&selected, &mut cache);
        assert_eq!(cache.len(), 2);
    }
}
