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
        pulls: Default::default(),
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

/// ankh with the work folders `folders` (id and machine), one group refining three
/// at once, and issues ankh#1 and ankh#2 in milestone "Clocks", not yet planned.
fn machines(folders: &[(&str, &str)]) -> World {
    let mut events = vec![Event::ProjectAdded {
        project: ProjectId::new("ankh"),
        tracker: "t".into(),
        name: "city/ankh".into(),
        code_host: None,
        repo: None,
    }];
    events.extend(
        folders
            .iter()
            .map(|(folder, machine)| work_folder(folder, machine)),
    );
    events.push(Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("ankh")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 3,
        model: None,
    });
    events.extend(["1", "2"].map(|key| Event::IssueSynced {
        issue: id("ankh", key),
        title: format!("ankh {key}"),
        milestone: Some(crate::ids::MilestoneId::new("Clocks")),
        depends_on: vec![],
        order: key.parse().unwrap_or_default(),
        closed: false,
    }));
    World::replay(&events).0
}

fn work_folder(folder: &str, machine: &str) -> Event {
    Event::FolderAdded {
        folder: FolderId::new(folder),
        project: ProjectId::new("ankh"),
        machine: MachineId::new(machine),
        path: format!("/srv/{folder}"),
    }
}

const BOTH: [(&str, &str); 2] = [("a-local", "local"), ("b-mac", "mac")];

fn machine(w: &mut World, scope: Scope, role: Option<Role>, on: &str) -> Vec<Effect> {
    w.apply(&Event::SettingSet {
        scope,
        role,
        key: "machine".into(),
        value: Value::Text(on.into()),
        by: "person".into(),
    })
}

fn ankh() -> Scope {
    Scope::Project(ProjectId::new("ankh"))
}

fn task(key: &str) -> Scope {
    Scope::Task(id("ankh", key))
}

fn plan(w: &mut World, key: &str) -> Vec<Effect> {
    w.apply(&Event::PlanWritten {
        issue: id("ankh", key),
        plan: crate::model::Plan::default(),
    })
}

fn author(key: &str, folder: &str) -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue: id("ankh", key),
        folder: Some(FolderId::new(folder)),
    }
}

/// The machine and folder path an agent of `role` on ankh#`key` would start in.
fn place_of(w: &World, role: Role, key: &str) -> Option<(String, String)> {
    let issue = id("ankh", key);
    let held = (role == Role::Author)
        .then(|| w.issues[&issue].folder.clone())
        .flatten();
    let launch = w.launch(&GroupId::new("watch"), role, &issue, held.as_ref())?;
    let path = launch.brief.folder?.display().to_string();
    Some((launch.machine.0, path))
}

fn at(machine: &str, folder: &str) -> Option<(String, String)> {
    Some((machine.into(), format!("/srv/{folder}")))
}

#[test]
fn an_author_takes_a_folder_on_its_issues_machine_and_any_folder_without_one() {
    let mut w = machines(&BOTH);
    machine(&mut w, task("1"), None, "mac");
    assert_eq!(plan(&mut w, "1"), vec![author("1", "b-mac")]);
    assert_eq!(place_of(&w, Role::Author, "1"), at("mac", "b-mac"));
    // Without the setting, the first free folder, as before.
    assert_eq!(plan(&mut w, "2"), vec![author("2", "a-local")]);
    let mut w = machines(&[("a-mac", "mac"), ("b-local", "local")]);
    assert_eq!(plan(&mut w, "1"), vec![author("1", "a-mac")]);
}

#[test]
fn a_machine_for_one_role_or_one_task_wins_where_the_chain_says() {
    let mut w = machines(&BOTH);
    machine(&mut w, ankh(), None, "mac");
    machine(&mut w, ankh(), Some(Role::Reviewer), "local");
    // The author reads the value for every role, the reviewer its own.
    assert_eq!(plan(&mut w, "1"), vec![author("1", "b-mac")]);
    assert_eq!(place_of(&w, Role::Reviewer, "1"), at("local", "a-local"));
    assert_eq!(place_of(&w, Role::Arbiter, "1"), at("mac", "b-mac"));
    // A value on the task is nearer than the project's, whatever its role.
    machine(&mut w, task("1"), None, "mac");
    assert_eq!(place_of(&w, Role::Reviewer, "1"), at("mac", "b-mac"));
    let r = w
        .setting(&id("ankh", "1"), Some(Role::Reviewer), "machine")
        .expect("set");
    assert_eq!(r.level, Level::Task);
    // A role's value for the author alone leaves the reviewer anywhere: it reads
    // from the folder that is free.
    let mut w = machines(&BOTH);
    machine(&mut w, ankh(), Some(Role::Author), "mac");
    assert_eq!(plan(&mut w, "1"), vec![author("1", "b-mac")]);
    assert_eq!(w.machine_for(&id("ankh", "1"), Role::Reviewer), None);
    assert_eq!(place_of(&w, Role::Reviewer, "1"), at("local", "a-local"));
}

#[test]
fn a_reader_shares_a_held_folder_on_its_machine_before_a_free_one_elsewhere() {
    let mut w = machines(&BOTH);
    machine(&mut w, ankh(), None, "mac");
    assert_eq!(plan(&mut w, "1"), vec![author("1", "b-mac")]);
    assert_eq!(place_of(&w, Role::Reviewer, "1"), at("mac", "b-mac"));
    assert_eq!(w.machine_waits(), vec![], "it can share the folder");
}

#[test]
fn an_issue_waits_ready_for_a_folder_on_its_machine_and_starts_when_one_is_released() {
    let mut w = machines(&[("a-local", "local"), ("b-mac", "mac"), ("c-local", "local")]);
    machine(&mut w, task("1"), None, "mac");
    machine(&mut w, task("2"), None, "mac");
    assert_eq!(plan(&mut w, "1"), vec![author("1", "b-mac")]);
    assert_eq!(plan(&mut w, "2"), vec![], "two local folders are free");
    let second = &w.issues[&id("ankh", "2")];
    assert_eq!((second.stage, &second.folder), (Stage::Ready, &None));
    assert!(
        w.folders
            .values()
            .all(|f| f.holder != Some(id("ankh", "2")))
    );
    assert!(w.missing().start.iter().all(|e| !matches!(
        e,
        Effect::StartAgent { issue, .. } if issue == &id("ankh", "2")
    )));
    assert_eq!(
        w.machine_waits(),
        vec![crate::placement::Waiting {
            issue: id("ankh", "2"),
            role: Role::Author,
            machine: MachineId::new("mac"),
            none_in: None,
        }]
    );
    // Another folder on this machine changes nothing.
    assert_eq!(w.apply(&work_folder("d-local", "local")), vec![]);
    // The first issue is closed, which releases the Mac folder.
    let effects = w.apply(&Event::IssueSynced {
        issue: id("ankh", "1"),
        title: "ankh 1".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    assert!(effects.contains(&author("2", "b-mac")), "{effects:?}");
    assert_eq!(w.machine_waits(), vec![]);
}

#[test]
fn a_machine_no_folder_is_on_keeps_every_agent_waiting_until_one_is() {
    let mut w = machines(&BOTH);
    // The refiner: the issue is not taken, and takes no refining slot.
    let mut fresh = machines(&BOTH);
    let milestone = Scope::Milestone {
        of: ProjectId::new("ankh"),
        name: "Clocks".into(),
    };
    machine(&mut fresh, milestone, Some(Role::Refiner), "mac2");
    let effects = fresh.apply(&Event::IssueSynced {
        issue: id("ankh", "3"),
        title: "ankh 3".into(),
        milestone: Some(crate::ids::MilestoneId::new("Clocks")),
        depends_on: vec![],
        order: 3,
        closed: false,
    });
    assert_eq!(effects, vec![]);
    assert_eq!(fresh.issues[&id("ankh", "3")].stage, Stage::New);
    let waits = fresh.machine_waits();
    let third = waits.iter().find(|w| w.issue == id("ankh", "3"));
    assert!(
        third.is_some_and(|w| w.role == Role::Refiner && w.none_in.is_some()),
        "{waits:?}"
    );

    // The author.
    machine(&mut w, task("1"), Some(Role::Author), "mac2");
    assert_eq!(plan(&mut w, "1"), vec![]);
    assert_eq!(w.issues[&id("ankh", "1")].stage, Stage::Ready);
    let waits = w.machine_waits();
    assert!(
        matches!(&waits[..], [w] if w.role == Role::Author && w.none_in.is_some()),
        "{waits:?}"
    );

    // The reviewer and the arbiter: the stage moves on, and no agent starts.
    let mut w = machines(&BOTH);
    set(&mut w, task("1"), None, "review_rounds", Value::Number(1));
    machine(&mut w, task("1"), Some(Role::Reviewer), "mac2");
    machine(&mut w, task("1"), Some(Role::Arbiter), "mac2");
    plan(&mut w, "1");
    let issue = id("ankh", "1");
    let review = w.apply(&Event::PullRequestReady {
        issue: issue.clone(),
        pull: Some("1".into()),
        pulls: Default::default(),
    });
    assert_eq!(review, vec![]);
    assert_eq!(place_of(&w, Role::Reviewer, "1"), None);
    let starts_for = |w: &World| -> Vec<Effect> {
        w.missing()
            .start
            .into_iter()
            .filter(|e| match e {
                Effect::StartReview { issue: i, .. } | Effect::StartAgent { issue: i, .. } => {
                    i == &issue
                }
                _ => false,
            })
            .collect()
    };
    assert_eq!(starts_for(&w), vec![]);
    assert_eq!(w.machine_waits()[0].role, Role::Reviewer);
    // A folder on that machine arrives, and the reviewer is asked for.
    w.apply(&work_folder("c-mac2", "mac2"));
    assert!(matches!(
        &starts_for(&w)[..],
        [Effect::StartReview { round: 1, .. }]
    ));
    assert_eq!(place_of(&w, Role::Reviewer, "1"), at("mac2", "c-mac2"));
    // Gone to another machine again: the arbiter waits the same way.
    machine(&mut w, task("1"), Some(Role::Arbiter), "klatch");
    let ruled = w.apply(&Event::ReviewFinished {
        issue: issue.clone(),
        round: 1,
        verdict: Verdict::Block {
            points: vec!["fix it".into()],
        },
    });
    assert!(
        !ruled.iter().any(|e| matches!(e, Effect::StartAgent { .. })),
        "{ruled:?}"
    );
    assert_eq!(starts_for(&w), vec![]);
    assert_eq!(w.machine_waits()[0].role, Role::Arbiter);
}

#[test]
fn an_issue_waiting_for_something_else_is_not_waiting_for_a_machine() {
    // One local folder, held: the second issue waits for a folder, with no setting.
    let mut w = machines(&[("a-local", "local")]);
    plan(&mut w, "1");
    assert_eq!(plan(&mut w, "2"), vec![]);
    assert_eq!(w.machine_waits(), vec![]);
    // A dependency is not done: that is what it waits for.
    let mut w = machines(&BOTH);
    w.apply(&Event::IssueSynced {
        issue: id("ankh", "2"),
        title: "ankh 2".into(),
        milestone: None,
        depends_on: vec![id("ankh", "1")],
        order: 2,
        closed: false,
    });
    machine(&mut w, task("2"), None, "mac2");
    machine(&mut w, task("2"), Some(Role::Refiner), "mac");
    plan(&mut w, "2");
    assert_eq!(w.machine_waits(), vec![]);
}

#[test]
fn an_issue_across_repositories_takes_all_its_folders_on_its_machine_or_none() {
    let project = |id: &str| Event::ProjectAdded {
        project: ProjectId::new(id),
        tracker: "t".into(),
        name: format!("city/{id}"),
        code_host: None,
        repo: None,
    };
    let folder = |folder: &str, repo: &str, machine: &str| Event::FolderAdded {
        folder: FolderId::new(folder),
        project: ProjectId::new(repo),
        machine: MachineId::new(machine),
        path: format!("/srv/{folder}"),
    };
    let issue = id("guild", "1");
    let (mut w, _) = World::replay(&[
        project("ui"),
        project("api"),
        Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Guild".into(),
            projects: vec![ProjectId::new("ui"), ProjectId::new("api")],
            tracker: "t".into(),
            tracker_name: "GLD".into(),
        },
        folder("ui-local", "ui", "local"),
        folder("ui-mac", "ui", "mac"),
        folder("api-local", "api", "local"),
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: Vec::new(),
            areas: vec![ProjectId::new("guild")],
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
        },
        Event::IssueSynced {
            issue: issue.clone(),
            title: "Both".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
    ]);
    machine(&mut w, Scope::Task(issue.clone()), None, "mac");
    let planned = w.apply(&Event::PlanWritten {
        issue: issue.clone(),
        plan: crate::model::Plan {
            repos: vec![ProjectId::new("ui"), ProjectId::new("api")],
            ..crate::model::Plan::default()
        },
    });
    // Both are free on this machine, but api has no folder on the Mac.
    assert_eq!(planned, vec![]);
    assert!(w.folders.values().all(|f| f.holder.is_none()));
    assert_eq!(w.machine_waits()[0].none_in, Some(ProjectId::new("api")));
    let effects = w.apply(&folder("api-mac", "api", "mac"));
    assert!(
        matches!(
            &effects[..],
            [Effect::StartAgent {
                role: Role::Author,
                ..
            }]
        ),
        "{effects:?}"
    );
    let held: Vec<&str> = w
        .folders
        .values()
        .filter(|f| f.holder.is_some())
        .map(|f| f.id.0.as_str())
        .collect();
    assert_eq!(held, ["api-mac", "ui-mac"]);
    assert_eq!(w.machine_waits(), vec![]);
}

#[test]
fn a_project_with_no_folder_runs_a_reader_here_unless_another_machine_is_named() {
    let mut w = machines(&[]);
    let launch = |w: &World| {
        w.launch(
            &GroupId::new("watch"),
            Role::Refiner,
            &id("ankh", "1"),
            None,
        )
        .map(|l| l.machine.0)
    };
    assert_eq!(launch(&w).as_deref(), Some("local"));
    machine(&mut w, ankh(), None, "local");
    assert_eq!(launch(&w).as_deref(), Some("local"));
    machine(&mut w, ankh(), None, "mac");
    assert_eq!(launch(&w), None);
}
