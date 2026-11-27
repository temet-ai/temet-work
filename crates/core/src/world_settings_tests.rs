//! A setting changes behaviour for exactly the scope it names.

use super::*;
use crate::event::{Event, Verdict};
use crate::ids::{GroupId, MachineId};
use crate::settings::{Level, Scope, Value};

fn id(project: &str, key: &str) -> IssueId {
    IssueId::new(project, key)
}

/// Two projects, ankh and klatch, each with a folder per issue, one group over both
/// that refines everything at once, and issues ankh#1, ankh#2 and klatch#1, all being
/// worked on.
fn working() -> World {
    let mut events = Vec::new();
    for project in ["ankh", "klatch"] {
        events.push(Event::ProjectAdded {
            project: ProjectId::new(project),
            tracker: "t".into(),
            name: format!("city/{project}"),
            code_host: None,
            repo: None,
        });
    }
    for (folder, project) in [("a1", "ankh"), ("a2", "ankh"), ("k1", "klatch")] {
        events.push(Event::FolderAdded {
            folder: FolderId::new(folder),
            project: ProjectId::new(project),
            machine: MachineId::new("morpork"),
            path: format!("/srv/{folder}"),
        });
    }
    events.push(Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("ankh"), ProjectId::new("klatch")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 3,
        model: None,
    });
    for (n, (project, key)) in [("ankh", "1"), ("ankh", "2"), ("klatch", "1")]
        .into_iter()
        .enumerate()
    {
        events.push(Event::IssueSynced {
            issue: id(project, key),
            title: format!("{project} {key}"),
            milestone: None,
            depends_on: vec![],
            order: n as u64,
            closed: false,
        });
    }
    for (project, key) in [("ankh", "1"), ("ankh", "2"), ("klatch", "1")] {
        events.push(Event::PlanWritten {
            issue: id(project, key),
            plan: crate::model::Plan::default(),
        });
    }
    let (w, _) = World::replay(&events);
    w
}

fn set(w: &mut World, scope: Scope, role: Option<Role>, key: &str, value: Value) {
    w.apply(&Event::SettingSet {
        scope,
        role,
        key: key.into(),
        value,
        by: "person".into(),
    });
}

/// Sends `issue` into review and has the reviewer block round `round`.
fn blocked(w: &mut World, issue: &IssueId, round: u8) -> Vec<Effect> {
    w.apply(&Event::PullRequestReady {
        issue: issue.clone(),
        pull: Some("1".into()),
    });
    w.apply(&Event::ReviewFinished {
        issue: issue.clone(),
        round,
        verdict: Verdict::Block {
            points: vec!["fix it".into()],
        },
    })
}

#[test]
fn review_rounds_on_one_issue_sends_only_that_issue_to_the_arbiter() {
    let mut w = working();
    set(
        &mut w,
        Scope::Task(id("ankh", "1")),
        None,
        "review_rounds",
        Value::Number(1),
    );
    let first = blocked(&mut w, &id("ankh", "1"), 1);
    assert!(
        first.iter().any(|e| matches!(
            e,
            Effect::StartAgent {
                role: Role::Arbiter,
                ..
            }
        )),
        "one round was all it had: {first:?}"
    );
    let neighbour = blocked(&mut w, &id("ankh", "2"), 1);
    assert!(
        neighbour
            .iter()
            .any(|e| matches!(e, Effect::RequestFix { .. })),
        "the neighbour still gets its second round: {neighbour:?}"
    );
    for round in 2..3 {
        let sent = blocked(&mut w, &id("ankh", "2"), round);
        assert!(sent.iter().any(|e| matches!(e, Effect::RequestFix { .. })));
    }
    let last = blocked(&mut w, &id("ankh", "2"), 3);
    assert!(last.iter().any(|e| matches!(
        e,
        Effect::StartAgent {
            role: Role::Arbiter,
            ..
        }
    )));
}

#[test]
fn a_model_on_a_project_reaches_the_brief_of_that_projects_issues_only() {
    let mut w = working();
    set(
        &mut w,
        Scope::Project(ProjectId::new("ankh")),
        None,
        "model",
        Value::Text("opus".into()),
    );
    let group = GroupId::new("watch");
    let model = |w: &World, project: &str| {
        w.launch(&group, Role::Reviewer, &id(project, "1"), None)
            .expect("known")
            .brief
            .model
    };
    assert_eq!(model(&w, "ankh").as_deref(), Some("opus"));
    assert_eq!(model(&w, "klatch"), None);
    // A task of its own beats the project, and only for that issue.
    set(
        &mut w,
        Scope::Task(id("ankh", "2")),
        None,
        "model",
        Value::Text("haiku".into()),
    );
    let launch = |w: &World, key: &str| {
        w.launch(&group, Role::Author, &id("ankh", key), None)
            .expect("known")
            .brief
            .model
    };
    assert_eq!(launch(&w, "2").as_deref(), Some("haiku"));
    assert_eq!(launch(&w, "1").as_deref(), Some("opus"));
}

#[test]
fn a_reviewers_only_runtime_reaches_reviewers_and_not_authors() {
    let mut w = working();
    set(
        &mut w,
        Scope::Global,
        Some(Role::Reviewer),
        "runtime",
        Value::Text("beta".into()),
    );
    let runtime = |role| w.setting_text(&id("ankh", "1"), Some(role), "runtime");
    assert_eq!(runtime(Role::Reviewer).as_deref(), Some("beta"));
    assert_eq!(runtime(Role::Author), None);
    assert_eq!(runtime(Role::Arbiter), None);
}

#[test]
fn starts_changes_how_many_agents_of_a_role_are_tried_on_one_issue() {
    let mut w = working();
    set(
        &mut w,
        Scope::Task(id("ankh", "1")),
        None,
        "starts",
        Value::Number(1),
    );
    let author = |n: u32| Event::AgentStarted {
        agent: crate::ids::AgentId::new(format!("author-{n}")),
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: Some(id("ankh", "1")),
        machine: MachineId::new("morpork"),
        folder: None,
    };
    for e in [author(1), author(2)] {
        w.apply(&e);
        w.apply(&Event::AgentChanged {
            agent: match &e {
                Event::AgentStarted { agent, .. } => agent.clone(),
                _ => unreachable!(),
            },
            state: crate::model::AgentState::Ended,
        });
    }
    // ankh#2 has had none and starts one; ankh#1 gave up after one.
    let missing = w.missing();
    assert_eq!(missing.given_up.len(), 1, "{missing:?}");
    assert!(missing.given_up[0].starts_with("ankh#1"));
    assert!(missing.start.iter().any(|e| matches!(
        e,
        Effect::StartAgent { issue, .. } if issue == &id("ankh", "2")
    )));
}

#[test]
fn with_no_settings_every_value_is_the_one_the_code_had() {
    let w = working();
    assert!(w.settings.is_empty());
    let issue = id("ankh", "1");
    let number = |key| w.setting_number(&issue, None, key);
    assert_eq!(number("review_rounds"), 3);
    assert_eq!(number("starts"), 3);
    assert_eq!(number("remind_after"), 10);
    assert_eq!(number("reminders"), 2);
    assert_eq!(number("sync_every"), 60);
    assert_eq!(number("ci_every"), 30);
    assert!(w.setting_flag(&issue, None, "ci"));
    assert_eq!(
        w.setting_text(&issue, None, "merge").as_deref(),
        Some("squash")
    );
    assert_eq!(
        w.setting_text(&issue, None, "permission").as_deref(),
        Some("auto")
    );
    for none in ["model", "deploy", "verify", "run_in"] {
        assert_eq!(w.setting(&issue, None, none), None, "{none}");
    }
    let launch = w
        .launch(&GroupId::new("watch"), Role::Author, &issue, None)
        .expect("known");
    assert_eq!(
        (launch.brief.model, launch.brief.permission.as_str()),
        (None, "auto")
    );
    // Three rounds, as before.
    let mut w = w;
    for round in 1..3 {
        let sent = blocked(&mut w, &issue, round);
        assert!(sent.iter().any(|e| matches!(e, Effect::RequestFix { .. })));
    }
    let last = blocked(&mut w, &issue, 3);
    assert!(last.iter().any(|e| matches!(
        e,
        Effect::StartAgent {
            role: Role::Arbiter,
            ..
        }
    )));
    let level = w.setting(&issue, None, "review_rounds").map(|r| r.level);
    assert_eq!(level, Some(Level::BuiltIn));
}

#[test]
fn an_older_log_with_commands_and_a_group_model_reads_as_settings() {
    let mut w = working();
    w.apply(&Event::ProjectCommandsSet {
        project: ProjectId::new("ankh"),
        deploy: Some("make deploy".into()),
        verify: None,
        run_in: Some("/srv".into()),
        ci: Some(false),
    });
    w.apply(&Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("ankh")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 1,
        model: Some("haiku".into()),
    });
    let i = id("ankh", "1");
    assert_eq!(
        w.setting_text(&i, None, "deploy").as_deref(),
        Some("make deploy")
    );
    assert_eq!(w.setting_text(&i, None, "run_in").as_deref(), Some("/srv"));
    assert!(!w.setting_flag(&i, None, "ci"));
    assert_eq!(w.setting_text(&i, None, "model").as_deref(), Some("haiku"));
    assert_eq!(w.setting_text(&id("klatch", "1"), None, "model"), None);
}
