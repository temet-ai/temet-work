use super::*;
use crate::command::Command;
use crate::event::Event;
use crate::ids::{FolderId, MachineId};
use crate::model::{AgentState, Plan};

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
                });
                started.push(launch);
            }
        }
    }
    started
}

/// Ends the running investigation as its session ending would, at `now_ms`.
fn end(w: &mut World, now_ms: u64) {
    let agent = w.investigating(&watch()).expect("running").agent.clone();
    w.apply(&Event::AgentChanged {
        agent,
        state: AgentState::Ended,
    });
    for e in w.investigation_events(now_ms, false) {
        w.apply(&e);
    }
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
    // A done issue with that title is no duplicate.
    let mut w = w;
    w.apply(&Event::IssueSynced {
        issue: IssueId::new("ankh", "1"),
        title: "issue 1".into(),
        milestone: Some(MilestoneId::new("First")),
        depends_on: vec![],
        order: 1,
        closed: true,
    });
    assert!(w.filing(Some(&me), &file("ankh", "issue 1", None)).is_ok());
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
