use super::*;
use crate::command::Command;
use crate::event::{Event, Verdict};
use crate::ids::{DecisionId, FolderId, MachineId, RuleId};
use crate::model::{AgentState, Plan, Rule, RuleState};

const MINUTE: u64 = 60_000;

fn watch() -> GroupId {
    GroupId::new("watch")
}

fn ankh() -> ProjectId {
    ProjectId::new("ankh")
}

fn project(id: &str, name: &str) -> Event {
    Event::ProjectAdded {
        project: ProjectId::new(id),
        tracker: "fake".into(),
        name: name.into(),
        code_host: None,
        repo: None,
    }
}

fn group(milestones: &[&str], queue_low: Option<usize>, investigate_every: Option<u64>) -> Event {
    Event::GroupCreated {
        group: watch(),
        name: "Night watch".into(),
        projects: vec![ankh()],
        areas: Vec::new(),
        milestones: milestones.iter().map(|m| MilestoneId::new(*m)).collect(),
        refining_at_once: 1,
        model: None,
        queue_low,
        investigate_every,
    }
}

fn folder(id: &str, project: &str) -> Event {
    Event::FolderAdded {
        folder: FolderId::new(id),
        project: ProjectId::new(project),
        machine: MachineId::new("morpork"),
        path: format!("/srv/{project}/{id}"),
    }
}

fn sync(key: &str, milestone: Option<&str>) -> Event {
    Event::IssueSynced {
        issue: IssueId::new("ankh", key),
        title: format!("issue {key}"),
        milestone: milestone.map(MilestoneId::new),
        depends_on: vec![],
        order: key.parse().unwrap_or(0),
        closed: false,
    }
}

/// A world with project ankh, one work folder of it, and the group watch.
fn short(queue_low: Option<usize>) -> World {
    World::replay(&[
        project("ankh", "city/ankh"),
        folder("f1", "ankh"),
        group(&["First"], queue_low, None),
    ])
    .0
}

/// What the host does at `now_ms`, with parts that start every investigator
/// asked for: applies the clock's events and starts each investigator the loop
/// asks for. Returns the investigators started.
fn tick(w: &mut World, now_ms: u64) -> Vec<Launch> {
    let mut started = Vec::new();
    for event in w.investigation_events(now_ms, true) {
        for effect in w.apply(&event) {
            if let Effect::Investigate { group } = effect {
                let launch = w.launch_investigator(&group).expect("a launch");
                w.apply(&Event::AgentStarted {
                    agent: launch.agent.clone(),
                    group: launch.group.clone(),
                    role: launch.brief.role,
                    issue: launch.brief.issue.clone(),
                    machine: launch.machine.clone(),
                    folder: launch.folder.clone(),
                    secret_digest: None,
                    runtime: None,
                    model: None,
                });
                started.push(launch);
            }
        }
    }
    started
}

/// Has the running investigator say it is done, as `temet-work investigated`.
fn investigated(w: &mut World) {
    let agent = w.investigating(&watch()).expect("running").agent.clone();
    w.apply(&Event::Investigated {
        group: watch(),
        agent,
    });
}

/// Ends the running investigation as its session ending would, at `now_ms`.
fn end(w: &mut World, now_ms: u64) {
    end_session(w, now_ms, false);
}

/// Ends the running investigation by its session ending, at `now_ms`, with its
/// program `lost` or not.
fn end_session(w: &mut World, now_ms: u64, lost: bool) {
    let agent = w.investigating(&watch()).expect("running").agent.clone();
    w.apply(&Event::AgentChanged {
        agent,
        state: AgentState::Ended,
        lost,
    });
    for e in w.investigation_events(now_ms, false) {
        w.apply(&e);
    }
}

/// The one investigator `tick` starts at `now_ms`.
fn next(w: &mut World, now_ms: u64) -> Launch {
    let mut started = tick(w, now_ms);
    assert_eq!(started.len(), 1, "{started:?}");
    started.remove(0)
}

/// The line of an investigator's brief that names its pull request window.
fn window_line(launch: &Launch) -> &str {
    launch
        .brief
        .task
        .lines()
        .find(|l| l.starts_with("3. "))
        .expect("line 3")
}

#[test]
fn a_group_below_queue_low_starts_one_investigator() {
    let mut w = short(None);
    assert_eq!(
        w.investigation_events(0, true),
        [Event::InvestigationStarted {
            group: watch(),
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        }]
    );
    let started = tick(&mut w, 0);
    let [launch] = started.as_slice() else {
        panic!("{started:?}");
    };
    assert_eq!(launch.agent, AgentId::new("investigator-1"));
    assert_eq!(launch.group, watch());
    assert_eq!(launch.brief.role, Role::Investigator);
    assert_eq!(launch.brief.issue, None, "it works on no issue");
    assert_eq!(launch.folder, None, "it holds no folder");
    let agent = &w.agents[&AgentId::new("investigator-1")];
    assert_eq!((agent.issue.as_ref(), agent.folder.as_ref()), (None, None));
    assert!(tick(&mut w, 1).is_empty(), "exactly one");
}

#[test]
fn untaken_new_issues_count_toward_the_queue() {
    // Issue 1 is refining, issue 2 waits as new in the group's milestone, and
    // issue 3 is outside it.
    let mut w = short(Some(2));
    for e in [
        sync("1", Some("First")),
        sync("2", Some("First")),
        sync("3", Some("Later")),
    ] {
        w.apply(&e);
    }
    assert_eq!(w.issues[&IssueId::new("ankh", "2")].stage, Stage::New);
    let g = w.groups[&watch()].clone();
    assert_eq!(w.queue(&g), 2, "refining 1 and new 2; not 3");
    assert!(tick(&mut w, 0).is_empty());
    // Taken by another group, a new issue is that group's work, not this one's.
    w.apply(&sync("2", Some("Later")));
    assert_eq!(w.queue(&g), 1);
    assert_eq!(tick(&mut w, 0).len(), 1);
}

/// A world with no work folder, so ready issues stay ready, whose group watch
/// (queue_low 8, milestone First) refined issues 1 to `first + later`, after which
/// the last `later` of them were synced into Later.
fn refined(first: u64, later: u64) -> World {
    let mut w = World::replay(&[
        project("ankh", "city/ankh"),
        group(&["First"], Some(8), None),
    ])
    .0;
    for key in 1..=first + later {
        w.apply(&sync(&key.to_string(), Some("First")));
        w.apply(&Event::PlanWritten {
            issue: IssueId::new("ankh", key.to_string()),
            plan: Plan::default(),
        });
    }
    for key in first + 1..=first + later {
        w.apply(&sync(&key.to_string(), Some("Later")));
    }
    assert!(w.issues.values().all(|i| i.stage == Stage::Ready));
    w
}

fn queued(w: &World) -> usize {
    w.queue(&w.groups[&watch()])
}

#[test]
fn out_of_scope_issues_leave_the_queue() {
    let w = refined(2, 10);
    assert_eq!(queued(&w), 2, "not 12");
    assert_eq!(
        w.investigation_events(0, true),
        [Event::InvestigationStarted {
            group: watch(),
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        }]
    );
    let enough = refined(8, 10);
    assert_eq!(queued(&enough), 8);
    assert_eq!(enough.investigation_events(0, true), []);
}

#[test]
fn out_of_scope_by_project_or_area_leaves_the_queue() {
    let takes = |projects: &[&str], areas: &[&str], milestones: &[&str]| Event::GroupCreated {
        group: watch(),
        name: "Night watch".into(),
        projects: projects.iter().map(|p| ProjectId::new(*p)).collect(),
        areas: areas.iter().map(|a| ProjectId::new(*a)).collect(),
        milestones: milestones.iter().map(|m| MilestoneId::new(*m)).collect(),
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    };
    let mut w = World::replay(&[
        project("ankh", "city/ankh"),
        Event::AreaDefined {
            area: ProjectId::new("guild"),
            name: "Thieves' Guild".into(),
            projects: vec![],
            tracker: "fake".into(),
            tracker_name: "GLD".into(),
        },
        takes(&["ankh"], &["guild"], &["First", "Second"]),
    ])
    .0;
    // A ready issue of the project, one of the area, and one still refining.
    for (source, key) in [("ankh", "1"), ("guild", "2")] {
        let issue = IssueId::new(source, key);
        w.apply(&Event::IssueSynced {
            issue: issue.clone(),
            title: format!("issue {key}"),
            milestone: Some(MilestoneId::new("First")),
            depends_on: vec![],
            order: key.parse().unwrap_or(0),
            closed: false,
        });
        w.apply(&Event::PlanWritten {
            issue,
            plan: Plan::default(),
        });
    }
    w.apply(&sync("3", Some("Second")));
    w.apply(&Event::AgentStarted {
        agent: AgentId::new("refiner-3"),
        group: watch(),
        role: Role::Refiner,
        issue: Some(IssueId::new("ankh", "3")),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    assert_eq!(queued(&w), 3);

    w.apply(&takes(&["ankh"], &["guild"], &["First"]));
    assert_eq!(
        w.issues[&IssueId::new("ankh", "3")].stage,
        Stage::Refining,
        "its refiner still runs"
    );
    assert_eq!(queued(&w), 2, "its milestone left the list");
    w.apply(&takes(&[], &["guild"], &["First"]));
    assert_eq!(queued(&w), 1, "its project left the list");
    w.apply(&takes(&[], &[], &["First"]));
    assert_eq!(queued(&w), 0, "its area left the list");
}

#[test]
fn issues_back_in_scope_count_again() {
    let mut w = refined(2, 10);
    let mut wider = w.clone();
    for key in 3..=8 {
        w.apply(&sync(&key.to_string(), Some("First")));
    }
    assert_eq!(queued(&w), 8);
    let status = crate::text::text(&w, &Default::default());
    assert!(status.contains("; 8 in its queue, "), "{status}");
    assert_eq!(w.investigating(&watch()), None);
    assert_eq!(w.investigation_events(0, true), []);

    wider.apply(&group(&["First", "Later"], Some(8), None));
    assert_eq!(queued(&wider), 12);
}

fn ankh_issue(key: &str) -> IssueId {
    IssueId::new("ankh", key)
}

fn sync_on(issue: &IssueId, milestone: Option<&str>, depends_on: &[&IssueId]) -> Event {
    Event::IssueSynced {
        issue: issue.clone(),
        title: format!("issue {}", issue.key),
        milestone: milestone.map(MilestoneId::new),
        depends_on: depends_on.iter().map(|d| (*d).clone()).collect(),
        order: issue.key.parse().unwrap_or(0),
        closed: false,
    }
}

/// Syncs an issue of ankh into First and has its refiner write the plan. No issue
/// may be refining already, since watch refines one at a time.
fn make_ready(w: &mut World, key: &str, depends_on: &[&IssueId]) {
    let issue = ankh_issue(key);
    w.apply(&sync_on(&issue, Some("First"), depends_on));
    w.apply(&Event::PlanWritten {
        issue: issue.clone(),
        plan: Plan::default(),
    });
    assert_eq!(w.issues[&issue].stage, Stage::Ready, "{issue}");
}

/// Project ankh with no work folder and its group watch (queue_low 8, milestone
/// First).
fn unstaffed() -> World {
    World::replay(&[
        project("ankh", "city/ankh"),
        group(&["First"], Some(8), None),
    ])
    .0
}

/// Every wait of a ready issue that would not start once a work folder is free.
fn start_waits(w: &World) -> Vec<crate::placement::StartWait> {
    w.issues
        .values()
        .filter(|i| i.stage == Stage::Ready)
        .flat_map(|i| w.start_waits(&w.groups[i.group.as_ref().expect("taken")], i))
        .collect()
}

/// Each wait as the waiting issue and the dependency it names; an empty one
/// when the issue is out of scope itself.
fn waits(w: &World) -> Vec<(String, String)> {
    start_waits(w)
        .into_iter()
        .map(|s| {
            let on = match s.why {
                crate::placement::Why::On { on, .. } => on.to_string(),
                crate::placement::Why::OutOfScope(_) => String::new(),
            };
            (s.issue.to_string(), on)
        })
        .collect()
}

fn wait(issue: &str, on: &str) -> (String, String) {
    (issue.into(), on.into())
}

fn status(w: &World) -> String {
    crate::text::text(w, &Default::default())
}

#[test]
fn ready_issues_on_an_out_of_scope_dependency_leave_the_queue() {
    let mut w = unstaffed();
    let later = ankh_issue("100");
    w.apply(&sync_on(&later, Some("Later"), &[]));
    for key in 1..=8 {
        make_ready(&mut w, &key.to_string(), &[&later]);
    }
    assert_eq!(queued(&w), 0);
    assert_eq!(
        w.investigation_events(0, true),
        [Event::InvestigationStarted {
            group: watch(),
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        }]
    );
    let status = status(&w);
    assert!(
        status.contains("; 0 in its queue, investigates below 8"),
        "{status}"
    );
    let lines: String = (1..=8)
        .map(|key| {
            format!(
                "  ankh#{key} waiting on ankh#100, out of scope \
                 (Later is not a milestone of watch)\n"
            )
        })
        .collect();
    assert!(
        status.contains(&format!("Waiting\n{lines}Agents\n")),
        "{status}"
    );
}

#[test]
fn a_dependency_no_group_covers_keeps_a_ready_issue_out_of_the_queue() {
    // In a project no group covers, with and without a milestone watch takes,
    // and in watch's own project with no milestone.
    let klatch = IssueId::new("klatch", "3");
    for (on, milestone) in [
        (&klatch, None),
        (&klatch, Some("First")),
        (&ankh_issue("100"), None),
    ] {
        let mut w = unstaffed();
        w.apply(&project("klatch", "desert/klatch"));
        w.apply(&sync_on(on, milestone, &[]));
        make_ready(&mut w, "1", &[on]);
        assert_eq!(queued(&w), 0, "{on} {milestone:?}");
        let status = status(&w);
        assert!(
            status.contains(&format!(
                "  ankh#1 waiting on {on}, out of scope (no group covers it)\n"
            )),
            "{status}"
        );
    }
}

#[test]
fn a_dependency_on_a_repository_nothing_matches_keeps_a_ready_issue_out_of_the_queue() {
    // The tracker keeps a name no project or area matches as the key of an issue
    // in the project that names it, and never syncs it.
    let (short, long) = (ankh_issue("nowhere#5"), ankh_issue("desert/elsewhere#12"));
    let mut w = unstaffed();
    for key in 1..=8 {
        make_ready(&mut w, &key.to_string(), &[&short]);
    }
    make_ready(&mut w, "9", &[&long]);
    make_ready(&mut w, "10", &[&ankh_issue("9")]);
    assert_eq!(queued(&w), 0);
    assert_eq!(w.investigation_events(0, true).len(), 1);
    let status = status(&w);
    for line in [
        "  ankh#1 waiting on nowhere#5, out of scope (no group covers it)\n",
        "  ankh#8 waiting on nowhere#5, out of scope (no group covers it)\n",
        "  ankh#9 waiting on desert/elsewhere#12, out of scope (no group covers it)\n",
        "  ankh#10 waiting on desert/elsewhere#12 through ankh#9, out of scope \
         (no group covers it)\n",
    ] {
        assert!(status.contains(line), "{line}{status}");
    }

    // Dropping the line counts the issue again.
    w.apply(&sync_on(&ankh_issue("1"), Some("First"), &[]));
    assert_eq!(queued(&w), 1);
}

#[test]
fn a_dependency_in_a_milestone_another_group_takes_keeps_it_out_too() {
    let mut w = unstaffed();
    let guard = GroupId::new("guard");
    w.apply(&Event::GroupCreated {
        group: guard.clone(),
        name: "Day watch".into(),
        projects: vec![ankh()],
        areas: Vec::new(),
        milestones: vec![MilestoneId::new("Later")],
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    let later = ankh_issue("100");
    w.apply(&sync_on(&later, Some("Later"), &[]));
    assert_eq!(w.issues[&later].group.as_ref(), Some(&guard));
    make_ready(&mut w, "1", &[&later]);
    assert_eq!(queued(&w), 0);
    assert_eq!(w.queue(&w.groups[&guard]), 1, "guard is refining ankh#100");
    assert_eq!(waits(&w), [wait("ankh#1", "ankh#100")]);
}

#[test]
fn done_dependencies_keep_a_ready_issue_in_the_queue() {
    let mut w = unstaffed();
    let (working, done, later) = (ankh_issue("7"), ankh_issue("50"), ankh_issue("100"));
    let unseen = ankh_issue("999");
    w.apply(&sync_on(&later, Some("Later"), &[]));
    w.apply(&Event::IssueSynced {
        issue: done.clone(),
        title: "issue 50".into(),
        milestone: Some(MilestoneId::new("Later")),
        depends_on: vec![],
        order: 50,
        closed: true,
    });
    // The one work folder goes to ankh#7, so the others stay ready.
    make_ready(&mut w, "7", &[]);
    w.apply(&folder("f1", "ankh"));
    make_ready(&mut w, "1", &[&working]);
    make_ready(&mut w, "2", &[&done]);
    make_ready(&mut w, "3", &[&unseen]);
    assert_eq!(w.issues[&working].stage, Stage::Working);
    assert_eq!(
        queued(&w),
        1,
        "2; 1 waits on 7, which is being worked on, and 3 on 999, never synced"
    );
    let waiting = [wait("ankh#1", "ankh#7"), wait("ankh#3", "ankh#999")];
    assert_eq!(waits(&w), waiting);

    // One refining and one new, both on the dependency in Later: they count.
    w.apply(&sync_on(&ankh_issue("4"), Some("First"), &[&later]));
    w.apply(&sync_on(&ankh_issue("5"), Some("First"), &[&later]));
    assert_eq!(w.issues[&ankh_issue("4")].stage, Stage::Refining);
    assert_eq!(w.issues[&ankh_issue("5")].stage, Stage::New);
    assert_eq!(queued(&w), 3);
    assert_eq!(waits(&w), waiting);
    let status = status(&w);
    assert!(
        status.contains(
            "Waiting\n  \
             ankh#1 waiting on ankh#7, not done (working)\n  \
             ankh#3 waiting on ankh#999, never synced\n\
             Agents\n"
        ),
        "{status}"
    );

    w.apply(&Event::PlanWritten {
        issue: ankh_issue("4"),
        plan: Plan::default(),
    });
    assert_eq!(queued(&w), 2, "4 is ready and cannot start");
    assert_eq!(
        waits(&w),
        [
            wait("ankh#1", "ankh#7"),
            wait("ankh#3", "ankh#999"),
            wait("ankh#4", "ankh#100")
        ]
    );
}

#[test]
fn closing_the_dependency_or_dropping_the_line_counts_the_issue_again() {
    let mut w = unstaffed();
    let later = ankh_issue("100");
    w.apply(&sync_on(&later, Some("Later"), &[]));
    make_ready(&mut w, "1", &[&later]);
    assert_eq!(queued(&w), 0);
    assert_eq!(waits(&w), [wait("ankh#1", "ankh#100")]);

    // Moved into scope, the dependency counts and the issue waits on it there.
    let mut moved = w.clone();
    moved.apply(&sync_on(&later, Some("First"), &[]));
    assert_eq!(queued(&moved), 1, "ankh#100, now refining");
    assert_eq!(waits(&moved), [wait("ankh#1", "ankh#100")]);
    assert!(
        status(&moved).contains("  ankh#1 waiting on ankh#100, not done (refining)\n"),
        "{}",
        status(&moved)
    );

    let mut closed = w.clone();
    closed.apply(&Event::IssueSynced {
        issue: later.clone(),
        title: "issue 100".into(),
        milestone: Some(MilestoneId::new("Later")),
        depends_on: vec![],
        order: 100,
        closed: true,
    });
    assert_eq!(queued(&closed), 1);

    let mut dropped = w;
    dropped.apply(&sync_on(&ankh_issue("1"), Some("First"), &[]));
    assert_eq!(queued(&dropped), 1);

    for w in [closed, dropped] {
        assert_eq!(waits(&w), []);
        assert!(!status(&w).contains("Waiting"), "{}", status(&w));
    }
}

#[test]
fn an_out_of_scope_dependency_through_another_issue_leaves_the_queue() {
    let mut w = unstaffed();
    let (near, far, later) = (ankh_issue("5"), ankh_issue("6"), ankh_issue("100"));
    w.apply(&sync_on(&later, Some("Later"), &[]));
    make_ready(&mut w, "5", &[&later]);
    make_ready(&mut w, "1", &[&near]);
    assert_eq!(queued(&w), 0);
    let chained = status(&w);
    assert!(
        chained.contains(
            "Waiting\n  \
             ankh#1 waiting on ankh#100 through ankh#5, out of scope \
             (Later is not a milestone of watch)\n  \
             ankh#5 waiting on ankh#100, out of scope (Later is not a milestone of watch)\n\
             Agents\n"
        ),
        "{chained}"
    );

    // Longer chains name each issue on the way, and one that closes on itself
    // still ends.
    make_ready(&mut w, "6", &[&near]);
    make_ready(&mut w, "2", &[&far]);
    make_ready(&mut w, "3", &[&ankh_issue("2")]);
    w.apply(&sync_on(&near, Some("First"), &[&ankh_issue("1"), &later]));
    assert_eq!(queued(&w), 0);
    let longer = status(&w);
    assert!(
        longer.contains("  ankh#2 waiting on ankh#100 through ankh#6 and ankh#5, out of scope"),
        "{longer}"
    );
    assert!(
        longer.contains(
            "  ankh#3 waiting on ankh#100 through ankh#2, ankh#6 and ankh#5, out of scope"
        ),
        "{longer}"
    );

    // A cycle with nothing out of scope waits on itself too: each of its two
    // issues names the other, and an issue that leads to it names the first of
    // them it meets.
    w.apply(&sync_on(&near, Some("First"), &[&ankh_issue("1")]));
    assert_eq!(queued(&w), 0);
    assert_eq!(
        waits(&w),
        [
            wait("ankh#1", "ankh#5"),
            wait("ankh#2", "ankh#5"),
            wait("ankh#3", "ankh#5"),
            wait("ankh#5", "ankh#1"),
            wait("ankh#6", "ankh#5"),
        ]
    );
    let cycle = status(&w);
    assert!(
        cycle.contains("  ankh#3 waiting on ankh#5 through ankh#2 and ankh#6, not done (ready)\n"),
        "{cycle}"
    );
}

/// The issues being worked on.
fn working(w: &World) -> Vec<String> {
    w.issues
        .values()
        .filter(|i| i.stage == Stage::Working)
        .map(|i| i.id.to_string())
        .collect()
}

/// The issues the Waiting section of `status` names.
fn named(status: &str) -> BTreeSet<String> {
    status
        .split_once("\nWaiting\n")
        .and_then(|(_, rest)| rest.split_once("\nAgents\n").or(Some((rest, ""))))
        .map(|(section, _)| section)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

#[test]
fn a_never_synced_dependency_keeps_a_ready_issue_out_of_the_queue() {
    // An issue of a known project, two of its own (#149's merged pull request
    // among them), and a name nothing matches; the world holds none of them.
    let mut w = unstaffed();
    w.apply(&project("klatch", "desert/klatch"));
    let never = [
        IssueId::new("klatch", "5"),
        ankh_issue("999"),
        ankh_issue("129"),
        ankh_issue("nowhere#5"),
    ];
    for (n, on) in never.iter().enumerate() {
        make_ready(&mut w, &(n + 1).to_string(), &[on]);
    }
    assert_eq!(queued(&w), 0);
    assert_eq!(
        waits(&w),
        [
            wait("ankh#1", "klatch#5"),
            wait("ankh#2", "ankh#999"),
            wait("ankh#3", "ankh#129"),
            wait("ankh#4", "ankh#nowhere#5"),
        ]
    );
    w.apply(&folder("f1", "ankh"));
    assert_eq!(working(&w), Vec::<String>::new());
    assert_eq!(w.folders[&FolderId::new("f1")].holder, None);
    assert_eq!(queued(&w), 0);
}

#[test]
fn the_status_names_a_never_synced_dependency() {
    let mut w = unstaffed();
    w.apply(&project("klatch", "desert/klatch"));
    make_ready(&mut w, "1", &[&IssueId::new("klatch", "5")]);
    make_ready(&mut w, "2", &[&ankh_issue("1")]);
    let status = crate::text::text(&w, &Default::default());
    for line in [
        "  ankh#1 waiting on klatch#5, never synced\n",
        "  ankh#2 waiting on klatch#5 through ankh#1, never synced\n",
    ] {
        assert!(status.contains(line), "{line}{status}");
    }
}

#[test]
fn a_chain_onto_a_never_synced_dependency_leaves_the_queue() {
    // ankh#2 needs ankh#1, which is in scope and ready, and ankh#1 needs
    // klatch#5, which the world never synced.
    let mut w = unstaffed();
    w.apply(&project("klatch", "desert/klatch"));
    make_ready(&mut w, "1", &[&IssueId::new("klatch", "5")]);
    make_ready(&mut w, "2", &[&ankh_issue("1")]);
    assert_eq!(queued(&w), 0);
    w.apply(&folder("f1", "ankh"));
    assert_eq!(working(&w), Vec::<String>::new());
    assert_eq!(
        waits(&w),
        [wait("ankh#1", "klatch#5"), wait("ankh#2", "klatch#5")]
    );
}

#[test]
fn eight_ready_issues_on_a_never_synced_dependency_start_an_investigator() {
    let mut w = unstaffed();
    w.apply(&project("klatch", "desert/klatch"));
    let unsynced = IssueId::new("klatch", "5");
    for key in 1..=8 {
        make_ready(&mut w, &key.to_string(), &[&unsynced]);
    }
    assert_eq!(
        w.investigation_events(0, true),
        [Event::InvestigationStarted {
            group: watch(),
            agent: AgentId::new("investigator-1"),
            at_ms: 0,
        }]
    );
    let status = status(&w);
    assert!(
        status.contains("; 0 in its queue, investigates below 8"),
        "{status}"
    );
    let lines: String = (1..=8)
        .map(|key| format!("  ankh#{key} waiting on klatch#5, never synced\n"))
        .collect();
    assert!(
        status.contains(&format!("Waiting\n{lines}Agents\n")),
        "{status}"
    );
}

#[test]
fn a_ready_issue_on_an_open_dependency_in_scope_waits() {
    let mut w = unstaffed();
    // ankh#7 takes the one work folder; ankh#6 waits for it, and for nothing else.
    make_ready(&mut w, "7", &[]);
    w.apply(&folder("f1", "ankh"));
    make_ready(&mut w, "6", &[]);
    let (seven, three, four) = (ankh_issue("7"), ankh_issue("3"), ankh_issue("4"));
    make_ready(&mut w, "1", &[&seven]);
    make_ready(&mut w, "2", &[&ankh_issue("1")]);
    make_ready(&mut w, "3", &[&four]);
    make_ready(&mut w, "4", &[&three]);
    make_ready(&mut w, "5", &[&ankh_issue("6")]);
    // One that names itself.
    make_ready(&mut w, "10", &[&ankh_issue("10")]);
    // On an issue that is refining and on one still new.
    make_ready(&mut w, "8", &[&ankh_issue("80")]);
    make_ready(&mut w, "9", &[&ankh_issue("90")]);
    w.apply(&sync_on(&ankh_issue("80"), Some("First"), &[]));
    w.apply(&sync_on(&ankh_issue("90"), Some("First"), &[]));
    assert_eq!(w.issues[&seven].stage, Stage::Working);
    assert_eq!(w.issues[&ankh_issue("80")].stage, Stage::Refining);
    assert_eq!(w.issues[&ankh_issue("90")].stage, Stage::New);
    assert_eq!(queued(&w), 3, "6, ready; 80, refining; 90, new");
    let status = status(&w);
    for line in [
        "  ankh#1 waiting on ankh#7, not done (working)\n",
        "  ankh#2 waiting on ankh#7 through ankh#1, not done (working)\n",
        "  ankh#3 waiting on ankh#4, not done (ready)\n",
        "  ankh#4 waiting on ankh#3, not done (ready)\n",
        "  ankh#5 waiting on ankh#6, not done (ready)\n",
        "  ankh#8 waiting on ankh#80, not done (refining)\n",
        "  ankh#9 waiting on ankh#90, not done (new)\n",
        "  ankh#10 waiting on ankh#10, not done (ready)\n",
    ] {
        assert!(status.contains(line), "{line}{status}");
    }
    assert_eq!(waits(&w).len(), 8, "{status}");
}

#[test]
fn a_ready_issue_out_of_scope_says_so_under_waiting() {
    let mut w = unstaffed();
    make_ready(&mut w, "3", &[]);
    assert_eq!(queued(&w), 1);
    w.apply(&sync_on(&ankh_issue("3"), Some("Later"), &[]));
    assert_eq!(queued(&w), 0);
    let later = status(&w);
    assert!(
        later.contains("  ankh#3  ready, out of scope  issue 3"),
        "{later}"
    );
    assert!(
        later.contains(
            "Waiting\n  ankh#3 waiting, out of scope (Later is not a milestone of watch)\nAgents\n"
        ),
        "{later}"
    );

    // With no milestone, and once the group no longer lists the project.
    w.apply(&sync_on(&ankh_issue("3"), None, &[]));
    let none = status(&w);
    assert!(
        none.contains("  ankh#3 waiting, out of scope (it has no milestone)\n"),
        "{none}"
    );
    w.apply(&project("klatch", "desert/klatch"));
    w.apply(&Event::GroupCreated {
        group: watch(),
        name: "Night watch".into(),
        projects: vec![ProjectId::new("klatch")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 1,
        model: None,
        queue_low: Some(8),
        investigate_every: None,
    });
    let moved = status(&w);
    assert!(
        moved.contains("  ankh#3 waiting, out of scope (ankh is not a project of watch)\n"),
        "{moved}"
    );

    // Another group that covers it takes it, and it waits no more.
    w.apply(&Event::GroupCreated {
        group: GroupId::new("guard"),
        name: "Day watch".into(),
        projects: vec![ankh()],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    assert_eq!(
        w.issues[&ankh_issue("3")].group,
        Some(GroupId::new("guard"))
    );
    assert!(!status(&w).contains("Waiting"), "{}", status(&w));
}

#[test]
fn every_ready_issue_starts_once_a_folder_is_free_or_says_why() {
    // watch covers ankh and lancre in First; no group covers klatch. ankh has no
    // work folder, and lancre#71 is being worked on in lancre's.
    let (mut w, _) = World::replay(&[
        project("ankh", "city/ankh"),
        project("lancre", "ramtops/lancre"),
        project("klatch", "desert/klatch"),
        folder("l1", "lancre"),
        Event::GroupCreated {
            group: watch(),
            name: "Night watch".into(),
            projects: vec![ankh(), ProjectId::new("lancre")],
            areas: Vec::new(),
            milestones: vec![MilestoneId::new("First")],
            refining_at_once: 1,
            model: None,
            queue_low: Some(8),
            investigate_every: None,
        },
    ]);
    let lancre = IssueId::new("lancre", "71");
    w.apply(&sync_on(&lancre, Some("First"), &[]));
    w.apply(&Event::PlanWritten {
        issue: lancre.clone(),
        plan: Plan::default(),
    });
    assert_eq!(w.issues[&lancre].stage, Stage::Working);
    w.apply(&sync_on(&ankh_issue("100"), Some("Later"), &[]));
    w.apply(&sync_on(&IssueId::new("klatch", "3"), None, &[]));
    w.apply(&Event::IssueSynced {
        issue: ankh_issue("50"),
        title: "issue 50".into(),
        milestone: Some(MilestoneId::new("First")),
        depends_on: vec![],
        order: 50,
        closed: true,
    });
    // One ready issue of each shape, by what it depends on.
    let shapes: [(&str, Option<IssueId>); 11] = [
        ("1", None),
        ("2", Some(ankh_issue("50"))),
        ("3", Some(ankh_issue("73"))),
        ("4", Some(ankh_issue("72"))),
        ("5", Some(ankh_issue("1"))),
        ("6", Some(lancre.clone())),
        ("7", Some(IssueId::new("klatch", "3"))),
        ("8", Some(ankh_issue("100"))),
        ("9", Some(ankh_issue("nowhere#5"))),
        ("10", Some(IssueId::new("klatch", "5"))),
        ("11", Some(ankh_issue("999"))),
    ];
    for (key, on) in &shapes {
        make_ready(&mut w, key, &on.iter().collect::<Vec<_>>());
    }
    // A chain through each of them, a cycle of two, and one out of scope.
    for (key, _) in &shapes {
        make_ready(&mut w, &format!("2{key}"), &[&ankh_issue(key)]);
    }
    make_ready(&mut w, "40", &[&ankh_issue("41")]);
    make_ready(&mut w, "41", &[&ankh_issue("40")]);
    make_ready(&mut w, "42", &[]);
    w.apply(&sync_on(&ankh_issue("42"), Some("Later"), &[]));
    make_ready(&mut w, "43", &[&ankh_issue("42")]);
    // Its dependencies that are refining and new come last: watch refines one at
    // a time.
    w.apply(&sync_on(&ankh_issue("72"), Some("First"), &[]));
    w.apply(&sync_on(&ankh_issue("73"), Some("First"), &[]));
    assert_eq!(w.issues[&ankh_issue("72")].stage, Stage::Refining);
    assert_eq!(w.issues[&ankh_issue("73")].stage, Stage::New);

    let ready: BTreeSet<String> = w
        .issues
        .values()
        .filter(|i| i.stage == Stage::Ready)
        .map(|i| i.id.to_string())
        .collect();
    assert_eq!(ready.len(), 2 * shapes.len() + 4);
    assert!(!w.folders.values().any(|f| f.project == ankh()));
    let before = named(&status(&w));
    let counted = queued(&w);

    for n in 0..ready.len() {
        w.apply(&folder(&format!("f{n}"), "ankh"));
    }
    let still: BTreeSet<String> = w
        .issues
        .values()
        .filter(|i| i.stage == Stage::Ready)
        .map(|i| i.id.to_string())
        .collect();
    let started: BTreeSet<String> = ready.difference(&still).cloned().collect();
    assert_eq!(
        started,
        BTreeSet::from(["ankh#1".to_string(), "ankh#2".to_string()]),
        "the one with no dependency and the one whose dependency is done"
    );
    assert_eq!(before, still, "{}", status(&w));
    assert_eq!(
        counted,
        started.len() + 2,
        "and ankh#72, refining, and ankh#73, new"
    );
}

#[test]
fn queue_low_defaults_to_twice_the_groups_work_folders() {
    let g = |w: &World| w.groups[&watch()].clone();
    let none = World::replay(&[project("ankh", "city/ankh"), group(&[], None, None)]).0;
    assert_eq!(none.queue_low(&g(&none)), 2, "no work folders");
    let three = World::replay(&[
        project("ankh", "city/ankh"),
        project("klatch", "desert/klatch"),
        folder("f1", "ankh"),
        folder("f2", "ankh"),
        folder("f3", "ankh"),
        folder("k1", "klatch"),
        group(&[], None, None),
    ])
    .0;
    assert_eq!(
        three.queue_low(&g(&three)),
        6,
        "klatch's folder is not the group's"
    );
    // The projects of the group's areas count too.
    let mut area = three.clone();
    area.apply(&Event::AreaDefined {
        area: ProjectId::new("desert"),
        name: "Desert".into(),
        projects: vec![ProjectId::new("klatch")],
        tracker: String::new(),
        tracker_name: String::new(),
    });
    area.apply(&Event::GroupCreated {
        group: watch(),
        name: "Night watch".into(),
        projects: vec![ankh()],
        areas: vec![ProjectId::new("desert")],
        milestones: vec![],
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    assert_eq!(area.queue_low(&g(&area)), 8);
    let set = World::replay(&[project("ankh", "city/ankh"), group(&[], Some(5), None)]).0;
    assert_eq!(set.queue_low(&g(&set)), 5, "the group's own value");
}

#[test]
fn the_investigator_holds_no_folder() {
    let mut w = short(Some(5));
    let started = tick(&mut w, 0);
    let launch = &started[0];
    assert_eq!(launch.folder, None);
    assert_eq!(
        launch.brief.folder,
        Some(std::path::PathBuf::from("/srv/ankh/f1")),
        "it runs in a free folder of the group's first project"
    );
    assert_eq!(launch.machine, MachineId::new("morpork"));
    assert_eq!(w.folders[&FolderId::new("f1")].holder, None);
    // So an author still gets the folder.
    w.apply(&sync("1", Some("First")));
    let effects = w.apply(&Event::PlanWritten {
        issue: IssueId::new("ankh", "1"),
        plan: Plan::default(),
    });
    assert!(
        effects.contains(&Effect::StartAgent {
            group: watch(),
            role: Role::Author,
            issue: IssueId::new("ankh", "1"),
            folder: Some(FolderId::new("f1")),
        }),
        "{effects:?}"
    );
}

#[test]
fn no_second_investigator_while_one_runs() {
    let mut w = short(None);
    assert_eq!(tick(&mut w, 0).len(), 1);
    for later in [1, 121 * MINUTE, 10_000 * MINUTE] {
        assert!(
            w.investigation_events(later, true).is_empty(),
            "still running at {later}"
        );
    }
    // Idle, it is waited on: reminded and then stopped like any agent whose step
    // is due.
    let agent = &w.agents[&AgentId::new("investigator-1")];
    assert!(w.waiting_on(agent));
}

#[test]
fn no_investigator_within_investigate_every_of_the_last_end() {
    let mut w = short(None);
    tick(&mut w, 0);
    end(&mut w, 10 * MINUTE);
    assert_eq!(
        w.investigations[&watch()].ended_ms,
        Some(10 * MINUTE),
        "measured from the end"
    );
    assert!(
        tick(&mut w, 129 * MINUTE).is_empty(),
        "two hours from the end have not passed"
    );
    let mut every = World::replay(&[
        project("ankh", "city/ankh"),
        group(&["First"], None, Some(30)),
    ])
    .0;
    tick(&mut every, 0);
    end(&mut every, 0);
    assert!(tick(&mut every, 30 * MINUTE - 1).is_empty());
}

#[test]
fn one_starts_again_after_investigate_every_when_still_short() {
    let mut w = short(None);
    tick(&mut w, 0);
    investigated(&mut w);
    end(&mut w, 10 * MINUTE);
    let started = tick(&mut w, 130 * MINUTE);
    assert_eq!(
        started
            .iter()
            .map(|l| l.agent.0.as_str())
            .collect::<Vec<_>>(),
        ["investigator-2"]
    );
    assert_eq!(w.investigations[&watch()].started_ms, 130 * MINUTE);
    assert_eq!(
        w.investigations[&watch()].since_ms,
        Some(0),
        "the last one's start"
    );
}

#[test]
fn an_unfinished_investigation_keeps_the_window_of_the_last_finished_one() {
    let mut w = short(None);
    next(&mut w, 0);
    investigated(&mut w);
    end(&mut w, 10 * MINUTE);
    next(&mut w, 130 * MINUTE);
    end_session(&mut w, 140 * MINUTE, true);
    let third = next(&mut w, 260 * MINUTE);
    assert_eq!(third.agent, AgentId::new("investigator-3"));
    assert_eq!(w.investigations[&watch()].since_ms, Some(0));
    assert_eq!(
        window_line(&third),
        "3. The pull requests merged since 1970-01-01 00:00 UTC, when the group's last \
         finished investigation started, for defects you can show in that work."
    );
}

#[test]
fn two_unfinished_investigations_in_a_row_do_not_move_the_window() {
    let mut w = short(None);
    next(&mut w, 0);
    investigated(&mut w);
    end(&mut w, 10 * MINUTE);
    next(&mut w, 130 * MINUTE);
    end_session(&mut w, 140 * MINUTE, true);
    next(&mut w, 260 * MINUTE);
    // Stopped, as an idle investigator is.
    end_session(&mut w, 270 * MINUTE, false);
    let fourth = next(&mut w, 390 * MINUTE);
    assert_eq!(fourth.agent, AgentId::new("investigator-4"));
    assert_eq!(w.investigations[&watch()].since_ms, Some(0));
    assert_eq!(
        window_line(&fourth),
        "3. The pull requests merged since 1970-01-01 00:00 UTC, when the group's last \
         finished investigation started, for defects you can show in that work."
    );
}

#[test]
fn an_unfinished_first_investigation_keeps_its_seven_days() {
    // 2026-11-27 13:05 UTC.
    let first = 1_795_784_700_000;
    let line = "3. The pull requests merged since 2026-11-20 13:05 UTC, 7 days before the \
                group's first investigation started, as none has finished yet, for defects you \
                can show in that work.";
    let mut w = short(None);
    next(&mut w, first);
    end(&mut w, first + 10 * MINUTE);
    let second = next(&mut w, first + 130 * MINUTE);
    assert_eq!(window_line(&second), line);
    end(&mut w, first + 140 * MINUTE);
    let third = next(&mut w, first + 260 * MINUTE);
    assert_eq!(window_line(&third), line);
}

#[test]
fn no_investigator_at_or_above_queue_low() {
    let mut w = short(Some(1));
    w.apply(&sync("1", Some("First")));
    assert!(
        tick(&mut w, 0).is_empty(),
        "one in the queue, one is the threshold"
    );
    let mut off = short(Some(0));
    assert!(tick(&mut off, 0).is_empty(), "0 starts none");
    // Below it again once the queue empties.
    w.apply(&Event::PlanWritten {
        issue: IssueId::new("ankh", "1"),
        plan: Plan::default(),
    });
    assert_eq!(w.issues[&IssueId::new("ankh", "1")].stage, Stage::Working);
    assert_eq!(tick(&mut w, 0).len(), 1);
}

#[test]
fn investigated_ends_the_investigation_and_stops_it() {
    let mut w = short(None);
    tick(&mut w, 0);
    let me = AgentId::new("investigator-1");
    let done = Command::Investigated { group: watch() };
    let events = w.events(Some(&me), &done).expect("accepted");
    assert_eq!(
        events,
        [Event::Investigated {
            group: watch(),
            agent: me.clone(),
        }]
    );
    assert_eq!(
        w.apply(&events[0]),
        [Effect::StopAgent { agent: me.clone() }]
    );
    assert!(!w.waiting_on(&w.agents[&me]), "done, so not reminded");
    assert_eq!(
        w.investigation_events(5, false),
        [Event::InvestigationEnded {
            group: watch(),
            at_ms: 5,
            filed: 0,
        }],
        "it ends even before its session is seen to stop"
    );
    for e in w.investigation_events(5, false) {
        w.apply(&e);
    }
    assert_eq!(w.investigating(&watch()), None);
    assert_eq!(w.investigations[&watch()].ended_ms, Some(5));
    // Nobody else may say so for it, and it may say so only once.
    let why = w.events(Some(&me), &done).expect_err("over");
    assert!(why.contains("is over"), "{why}");
    let mut w = short(None);
    tick(&mut w, 0);
    let person = w.events(None, &done).expect_err("a person");
    assert!(person.contains("a person may not run it"), "{person}");
    let other = w
        .events(
            Some(&me),
            &Command::Investigated {
                group: GroupId::new("guild"),
            },
        )
        .expect_err("another group");
    assert!(other.contains("not guild"), "{other}");
}

#[test]
fn an_ended_investigator_ends_the_investigation() {
    let mut w = short(None);
    tick(&mut w, 0);
    w.apply(&Event::IssueFiled {
        group: watch(),
        agent: AgentId::new("investigator-1"),
        issue: IssueId::new("ankh", "7"),
        title: "Wind the clock".into(),
        milestone: Some(MilestoneId::new("First")),
    });
    w.apply(&Event::AgentChanged {
        agent: AgentId::new("investigator-1"),
        state: AgentState::Ended,
        lost: false,
    });
    assert_eq!(
        w.investigation_events(7, false),
        [Event::InvestigationEnded {
            group: watch(),
            at_ms: 7,
            filed: 1,
        }]
    );
}

#[test]
fn an_investigation_whose_investigator_never_started_asks_for_it_again() {
    let mut w = short(None);
    for e in w.investigation_events(0, true) {
        assert_eq!(w.apply(&e), [Effect::Investigate { group: watch() }]);
    }
    assert_eq!(
        w.missing().start,
        [Effect::Investigate { group: watch() }],
        "the start was lost"
    );
    let launch = w.launch_investigator(&watch()).expect("launch");
    assert_eq!(
        launch.agent,
        AgentId::new("investigator-1"),
        "the id it recorded"
    );
}

#[test]
fn two_groups_short_at_once_get_investigators_of_their_own() {
    let mut w = short(None);
    w.apply(&Event::GroupCreated {
        group: GroupId::new("guard"),
        name: "Day guard".into(),
        projects: vec![ankh()],
        areas: vec![],
        milestones: vec![],
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    let started = tick(&mut w, 0);
    let ids: Vec<(&str, &str)> = started
        .iter()
        .map(|l| (l.group.0.as_str(), l.agent.0.as_str()))
        .collect();
    assert_eq!(
        ids,
        [("guard", "investigator-1"), ("watch", "investigator-2")]
    );
}

/// A world where investigator-1 runs for watch, which takes milestones First and
/// Second of ankh, and klatch is a project no group takes.
fn investigating() -> World {
    let mut w = World::replay(&[
        project("ankh", "city/ankh"),
        project("klatch", "desert/klatch"),
        folder("f1", "ankh"),
        group(&["First", "Second"], None, None),
        sync("1", Some("First")),
    ])
    .0;
    tick(&mut w, 0);
    w.apply(&Event::AgentStarted {
        agent: AgentId::new("refiner-9"),
        group: watch(),
        role: Role::Refiner,
        issue: Some(IssueId::new("ankh", "1")),
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
    });
    w
}

fn file(project: &str, title: &str, milestone: Option<&str>) -> Command {
    Command::File {
        project: ProjectId::new(project),
        title: title.into(),
        body: "Depends on #1".into(),
        milestone: milestone.map(String::from),
    }
}

#[test]
fn file_takes_the_groups_last_milestone_by_default() {
    let w = investigating();
    let me = AgentId::new("investigator-1");
    assert_eq!(
        w.filing(Some(&me), &file("ankh", "Wind the clock", None)),
        Ok(Effect::File {
            group: watch(),
            agent: me.clone(),
            source: ankh(),
            title: "Wind the clock".into(),
            body: "Depends on #1".into(),
            milestone: Some(MilestoneId::new("Second")),
        })
    );
    let named = w
        .filing(Some(&me), &file("ankh", "Wind the clock", Some("First")))
        .expect("a milestone the group takes");
    assert!(matches!(named, Effect::File { milestone: Some(m), .. } if m.0 == "First"));
    // The command becomes no event of its own: the daemon files it and logs what
    // the tracker answered.
    assert_eq!(
        w.events(Some(&me), &file("ankh", "Wind the clock", None)),
        Ok(vec![])
    );
    // A group that names no milestone files with none.
    let mut every = World::replay(&[project("ankh", "city/ankh"), group(&[], None, None)]).0;
    tick(&mut every, 0);
    assert!(matches!(
        every.filing(Some(&me), &file("ankh", "Wind the clock", None)),
        Ok(Effect::File {
            milestone: None,
            ..
        })
    ));
}

#[test]
fn file_is_refused_for_a_person_and_for_another_agent() {
    let w = investigating();
    let command = file("ankh", "Wind the clock", None);
    let person = w.filing(None, &command).expect_err("a person");
    assert!(person.contains("a person may not run it"), "{person}");
    let refiner = w
        .filing(Some(&AgentId::new("refiner-9")), &command)
        .expect_err("a refiner");
    assert!(
        refiner.contains("refiner-9 is the refiner of ankh#1"),
        "{refiner}"
    );
    let stranger = w
        .filing(Some(&AgentId::new("investigator-7")), &command)
        .expect_err("not started");
    assert!(
        stranger.contains("not an agent this daemon started"),
        "{stranger}"
    );
    // An investigator whose investigation is over files no more.
    let mut over = w.clone();
    end(&mut over, 1);
    let ended = over
        .filing(Some(&AgentId::new("investigator-1")), &command)
        .expect_err("over");
    assert!(ended.contains("is over"), "{ended}");
}

#[test]
fn file_is_refused_for_a_project_the_group_does_not_take() {
    let w = investigating();
    let why = w
        .filing(
            Some(&AgentId::new("investigator-1")),
            &file("klatch", "Wind the clock", None),
        )
        .expect_err("klatch");
    assert_eq!(why, "watch takes no issues from klatch; it files in: ankh");
}

#[test]
fn file_is_refused_for_a_milestone_the_group_does_not_take() {
    let w = investigating();
    let why = w
        .filing(
            Some(&AgentId::new("investigator-1")),
            &file("ankh", "Wind the clock", Some("Later")),
        )
        .expect_err("Later");
    assert_eq!(
        why,
        "watch does not take milestone Later; it takes: First, Second"
    );
}

#[test]
fn file_is_refused_for_an_open_issue_with_the_same_title() {
    let w = investigating();
    let me = AgentId::new("investigator-1");
    let why = w
        .filing(Some(&me), &file("ankh", "  Issue 1 ", None))
        .expect_err("a duplicate");
    assert!(
        why.starts_with("ankh#1 is open with the title \"issue 1\""),
        "{why}"
    );
    let empty = w
        .filing(Some(&me), &file("ankh", " ", None))
        .expect_err("no title");
    assert_eq!(empty, "an issue needs a title");
    // An issue the loop closed after verifying it is no duplicate.
    let mut w = w;
    close_by_loop(&mut w, "ankh", "1");
    assert!(w.filing(Some(&me), &file("ankh", "issue 1", None)).is_ok());
}

/// Syncs `project#key` as the tracker lists it, `closed` or open.
fn titled(w: &mut World, project: &str, key: &str, title: &str, order: u64, closed: bool) {
    w.apply(&Event::IssueSynced {
        issue: IssueId::new(project, key),
        title: title.into(),
        milestone: None,
        depends_on: vec![],
        order,
        closed,
    });
}

/// A person closes `project#key` in the tracker after the loop saw it open.
fn close_by_person(w: &mut World, project: &str, key: &str, title: &str, order: u64) {
    titled(w, project, key, title, order, false);
    titled(w, project, key, title, order, true);
}

/// The loop closes `project#key`, which it holds, after verifying it.
fn close_by_loop(w: &mut World, project: &str, key: &str) {
    let issue = IssueId::new(project, key);
    w.issues.get_mut(&issue).expect("held").stage = Stage::Verifying;
    w.apply(&Event::Verified {
        issue: issue.clone(),
        passed: true,
        output: String::new(),
    });
    assert!(w.issues[&issue].closed_by_loop, "{issue}");
}

fn brief_of(w: &World) -> String {
    w.launch_investigator(&watch()).expect("launch").brief.task
}

/// The line a brief introduces its list of the issues a person closed with.
const CLOSED_HEAD: &str = "A person closed these issues. Never file one of them again, because \
                           closing an issue is how a person says no, and a person who wants it \
                           built reopens it:";

/// The entries of `brief`'s list of the issues a person closed, when it has one.
fn closed_list(brief: &str) -> Option<Vec<&str>> {
    let mut lines = brief.lines().skip_while(|l| *l != CLOSED_HEAD);
    lines.next()?;
    Some(lines.take_while(|l| l.starts_with("- ")).collect())
}

#[test]
fn the_brief_lists_the_issues_a_person_closed() {
    let mut w = investigating();
    let none = brief_of(&w);
    assert_eq!(closed_list(&none), None, "{none}");
    assert!(!none.contains("a person closed"), "{none}");

    // The tracker's order, not the keys': 3 is older than 12, whose key sorts
    // first as text.
    close_by_person(&mut w, "ankh", "12", "Oil the gate", 30);
    close_by_person(&mut w, "ankh", "3", "Wind the clock", 20);
    // Closed by the loop after verifying.
    titled(&mut w, "ankh", "4", "Sweep the yard", 40, false);
    close_by_loop(&mut w, "ankh", "4");
    // A closed dependency the loop never saw open.
    titled(&mut w, "ankh", "5", "", 0, true);
    // A project the group does not take.
    close_by_person(&mut w, "klatch", "6", "Roast the beans", 10);
    // Closed and then reopened.
    close_by_person(&mut w, "ankh", "8", "Paint the door", 50);
    titled(&mut w, "ankh", "8", "Paint the door", 50, false);

    let brief = brief_of(&w);
    assert_eq!(
        closed_list(&brief),
        Some(vec!["- ankh#3 Wind the clock", "- ankh#12 Oil the gate"]),
        "{brief}"
    );
    for not in [
        "Sweep the yard",
        "ankh#5",
        "Roast the beans",
        "Paint the door",
    ] {
        assert!(!brief.contains(not), "{not:?} is in:\n{brief}");
    }
    for line in [
        "2. The open issues of each project, and the ones a person closed, listed below, so you \
         know what is covered already and what was turned down.",
        "Never file an issue that an open one, or one a person closed, already covers. Change no \
         file.",
    ] {
        assert!(brief.contains(line), "missing {line:?} in:\n{brief}");
    }
}

#[test]
fn the_brief_lists_a_persons_closes_in_the_groups_areas() {
    let mut w = investigating();
    w.apply(&Event::AreaDefined {
        area: ProjectId::new("guild"),
        name: "Thieves' Guild".into(),
        projects: vec![ProjectId::new("klatch")],
        tracker: "fake".into(),
        tracker_name: "GLD".into(),
    });
    w.apply(&Event::GroupCreated {
        group: watch(),
        name: "Night watch".into(),
        projects: vec![ankh()],
        areas: vec![ProjectId::new("guild")],
        milestones: vec![MilestoneId::new("First")],
        refining_at_once: 1,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    // Listed whatever its milestone: these have none.
    close_by_person(&mut w, "guild", "2", "Count the purses", 1);
    close_by_person(&mut w, "klatch", "6", "Roast the beans", 2);
    let brief = brief_of(&w);
    assert_eq!(
        closed_list(&brief),
        Some(vec![
            "- guild#2 Count the purses",
            "- klatch#6 Roast the beans"
        ]),
        "{brief}"
    );
}

#[test]
fn file_is_refused_for_an_issue_a_person_closed() {
    let mut w = investigating();
    let me = AgentId::new("investigator-1");
    close_by_person(&mut w, "ankh", "3", "Wind the clock", 3);
    let why = w
        .filing(Some(&me), &file("ankh", "  wind the CLOCK ", None))
        .expect_err("a person said no");
    assert_eq!(
        why,
        "ankh#3 was closed by a person with the title \"Wind the clock\"; do not file it again: a \
         person who wants it built reopens it"
    );
    // The same title of an issue the loop closed after verifying it is filed.
    titled(&mut w, "ankh", "4", "Sweep the yard", 4, false);
    close_by_loop(&mut w, "ankh", "4");
    assert!(
        w.filing(Some(&me), &file("ankh", "Sweep the yard", None))
            .is_ok()
    );
    // A close in another project says nothing about this one.
    close_by_person(&mut w, "klatch", "6", "Roast the beans", 6);
    assert!(
        w.filing(Some(&me), &file("ankh", "Roast the beans", None))
            .is_ok()
    );
}

#[test]
fn an_issue_closed_after_its_pull_request_merged_is_not_a_persons_close() {
    let mut w = short(None);
    tick(&mut w, 0);
    let me = AgentId::new("investigator-1");
    let issue = IssueId::new("ankh", "7");
    let sync = |w: &mut World, closed| {
        titled(w, "ankh", "7", "Mend the bridge", 7, closed);
    };
    w.apply(&Event::IssueSynced {
        issue: issue.clone(),
        title: "Mend the bridge".into(),
        milestone: Some(MilestoneId::new("First")),
        depends_on: vec![],
        order: 7,
        closed: false,
    });
    for e in [
        Event::PlanWritten {
            issue: issue.clone(),
            plan: Plan::default(),
        },
        Event::PullRequestReady {
            issue: issue.clone(),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        },
        Event::ReviewFinished {
            issue: issue.clone(),
            round: 1,
            verdict: Verdict::Approve,
            failed: None,
            open: None,
        },
        Event::ChecksFinished {
            issue: issue.clone(),
            passed: true,
            failing: vec![],
        },
        Event::Merged {
            issue: issue.clone(),
            repo: None,
        },
        Event::Deployed {
            issue: issue.clone(),
        },
    ] {
        w.apply(&e);
    }
    assert_eq!(w.issues[&issue].stage, Stage::Verifying);
    // The pull request closed the issue as it merged; the sync is ignored while
    // the issue is verifying.
    sync(&mut w, true);
    assert_eq!(w.issues[&issue].stage, Stage::Verifying);
    w.apply(&Event::Verified {
        issue: issue.clone(),
        passed: false,
        output: "the bridge is still down".into(),
    });
    assert!(w.issues[&issue].merged.is_empty());
    sync(&mut w, true);
    assert_eq!(w.issues[&issue].stage, Stage::Done);
    assert!(!w.issues[&issue].closed_by_loop);

    let brief = brief_of(&w);
    assert_eq!(closed_list(&brief), None, "{brief}");
    assert!(!brief.contains("Mend the bridge"), "{brief}");
    assert!(
        w.filing(Some(&me), &file("ankh", "Mend the bridge", None))
            .is_ok()
    );

    // Reopened, it starts again, and a close of it then is a person's.
    sync(&mut w, false);
    sync(&mut w, true);
    let brief = brief_of(&w);
    assert_eq!(
        closed_list(&brief),
        Some(vec!["- ankh#7 Mend the bridge"]),
        "{brief}"
    );
}

#[test]
fn a_person_closed_issue_stays_refused_in_later_investigations() {
    let mut w = short(None);
    close_by_person(&mut w, "ankh", "3", "Wind the clock", 3);
    tick(&mut w, 0);
    investigated(&mut w);
    end(&mut w, 10 * MINUTE);
    let started = tick(&mut w, 130 * MINUTE);
    let [second] = started.as_slice() else {
        panic!("{started:?}");
    };
    assert_eq!(second.agent, AgentId::new("investigator-2"));
    assert_eq!(w.investigations[&watch()].since_ms, Some(0));
    assert_eq!(
        closed_list(&second.brief.task),
        Some(vec!["- ankh#3 Wind the clock"])
    );
    let why = w
        .filing(Some(&second.agent), &file("ankh", "Wind the clock", None))
        .expect_err("still refused");
    assert!(
        why.starts_with("ankh#3 was closed by a person with the title"),
        "{why}"
    );
}

/// Keeps 20 rules of `bytes` bytes each, and returns their texts.
fn keep_rules(w: &mut World, bytes: usize) -> Vec<String> {
    (1..=20)
        .map(|n| {
            let text = format!("{:x<bytes$}", format!("Prefer rule {n:02} "));
            let id = RuleId(n);
            w.rules.insert(
                id,
                Rule {
                    id,
                    decision: DecisionId(n),
                    issue: ankh_issue("1"),
                    decided_by: AgentId::new("refiner-1"),
                    overturned_by: "person".into(),
                    text: text.clone(),
                    state: RuleState::Kept,
                },
            );
            text
        })
        .collect()
}

#[test]
fn a_long_closed_list_leaves_the_kept_rules_their_room() {
    let mut w = investigating();
    // 110 KB of titles. The tracker's order runs against the keys'.
    // Each is a copy of one a person closed, since 2,200 syncs would take
    // seconds.
    close_by_person(&mut w, "ankh", "3", "Wind the clock", 3);
    let one = w.issues.remove(&ankh_issue("3")).expect("closed");
    let entries: Vec<String> = (0..1100)
        .map(|n| {
            let mut issue = one.clone();
            issue.id = ankh_issue(&(2100 - n).to_string());
            issue.title = format!("{:-<100}", format!("Wind {} ", issue.id));
            assert_eq!(issue.title.len(), 100);
            issue.order = n;
            let entry = format!("- {} {}", issue.id, issue.title);
            w.issues.insert(issue.id.clone(), issue);
            entry
        })
        .collect();
    for (bytes, printed) in [(2048, 1000), (900, 900)] {
        let rules = keep_rules(&mut w, bytes);
        let brief = brief_of(&w);
        assert!(brief.len() <= crate::fit::LIMIT, "{} bytes", brief.len());

        let (_, kept) = brief
            .split_once("Rules from overturned decisions:\n")
            .expect("the rules");
        let pointer = if printed < bytes {
            " (cut here; temet-work status prints it whole, under Rules)"
        } else {
            ""
        };
        for rule in &rules {
            let line = format!("- {}{pointer}\n", &rule[..printed]);
            assert!(kept.contains(&line), "{bytes}: missing {line:?}");
        }
        for left in ["earlier rules are left out", "earlier rule is left out"] {
            assert!(!brief.contains(left), "{bytes}: {left}");
        }

        // The first entries that fit, each whole, and then the line counting
        // the others.
        let listed = closed_list(&brief).expect("the list");
        let n = listed.len();
        assert!(n > 0 && n < entries.len(), "{bytes}: {n} listed");
        assert_eq!(listed, entries[..n], "{bytes}");
        assert_eq!(
            brief.lines().filter(|l| l.starts_with("- ankh#")).count(),
            n,
            "{bytes}"
        );
        let next = brief.lines().skip_while(|l| *l != listed[n - 1]).nth(1);
        let left = format!(
            "{} more issues a person closed are left out here; temet-work closed watch prints \
             every one.",
            entries.len() - n
        );
        assert_eq!(next, Some(left.as_str()), "{bytes}");
        // No room for another entry is left unused.
        assert!(
            brief.len() + entries[n].len() >= crate::fit::LIMIT,
            "{bytes}: {} bytes",
            brief.len()
        );
    }
}

#[test]
fn a_closed_title_is_cleaned_indented_and_cut_as_any_item() {
    let mut w = investigating();
    close_by_person(&mut w, "ankh", "3", "Wind\x1b\0 it\r\n- ankh#99 Fake", 3);
    close_by_person(&mut w, "ankh", "4", &"y".repeat(2000), 4);
    let brief = brief_of(&w);
    let cut = format!(
        "- ankh#4 {} (cut here; temet-work closed watch prints it whole)\n",
        "y".repeat(993)
    );
    let list = format!("{CLOSED_HEAD}\n- ankh#3 Wind it\n  - ankh#99 Fake\n{cut}");
    assert!(brief.contains(&list), "{brief}");
}

#[test]
fn a_filed_issue_counts_toward_the_queue_and_reaches_the_group() {
    let mut w = investigating();
    let g = w.groups[&watch()].clone();
    assert_eq!(w.queue(&g), 1);
    let effects = w.apply(&Event::IssueSynced {
        issue: IssueId::new("ankh", "2"),
        title: "Wind the clock".into(),
        milestone: Some(MilestoneId::new("Second")),
        depends_on: vec![IssueId::new("ankh", "1")],
        order: 2,
        closed: false,
    });
    assert!(
        effects.is_empty(),
        "one refines at a time; it waits its turn"
    );
    assert_eq!(w.queue(&g), 2, "it counts at once");
    w.apply(&Event::IssueFiled {
        group: watch(),
        agent: AgentId::new("investigator-1"),
        issue: IssueId::new("ankh", "2"),
        title: "Wind the clock".into(),
        milestone: Some(MilestoneId::new("Second")),
    });
    assert_eq!(
        w.investigations[&watch()].filed,
        [IssueId::new("ankh", "2")]
    );
}

#[test]
fn an_agent_may_not_set_investigate() {
    let w = investigating();
    let set = Command::Set {
        target: crate::settings::Target::parse("ankh#1"),
        role: None,
        key: "investigate".into(),
        value: "The clock tower is the Guild's".into(),
    };
    let why = w
        .events(Some(&AgentId::new("refiner-9")), &set)
        .expect_err("an agent");
    assert_eq!(
        why,
        "only a person sets investigate, because it is text written into every later \
         investigator's brief"
    );
    let person = Command::Set {
        target: crate::settings::Target::parse("ankh"),
        role: None,
        key: "investigate".into(),
        value: "The clock tower is the Guild's".into(),
    };
    assert!(w.events(None, &person).is_ok());
}

#[test]
fn a_projects_investigate_lines_end_up_in_the_brief() {
    let mut w = investigating();
    for (scope, text) in [
        (
            Scope::Project(ankh()),
            "The clock tower is the Guild's: file nothing for it.",
        ),
        (Scope::Global, "Ask before filing for Unseen University."),
    ] {
        w.apply(&Event::SettingSet {
            scope,
            role: None,
            key: "investigate".into(),
            value: crate::settings::Value::Text(text.into()),
            by: "person".into(),
        });
    }
    let task = w.launch_investigator(&watch()).expect("launch").brief.task;
    assert!(
        task.contains(
            "What the projects add:\nFor ankh:\nThe clock tower is the Guild's: file nothing for it.\n"
        ),
        "the project's value, under its name: {task}"
    );
    assert!(
        !task.contains("Unseen"),
        "the project's value is nearer than global: {task}"
    );
}

#[test]
fn utc_writes_a_date_and_time() {
    assert_eq!(utc(0), "1970-01-01 00:00 UTC");
    // 27 November 2026, 13:05 UTC.
    assert_eq!(utc(1_795_784_700_000), "2026-11-27 13:05 UTC");
    assert_eq!(utc(951_825_600_000), "2000-02-29 12:00 UTC");
}
