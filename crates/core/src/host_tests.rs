use std::sync::{Arc, Mutex};

use super::*;
use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, MilestoneId, ProjectId};
use crate::launch::Program;
use crate::model::{AgentState, Plan, Role};

/// A journal the test can read after the host has taken it.
#[derive(Clone, Default)]
struct Shared(Arc<Mutex<Vec<Event>>>, bool);

impl Journal for Shared {
    fn append(&mut self, _at_ms: u64, event: &Event) -> Result<(), String> {
        if self.1 {
            return Err("disk full".into());
        }
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push(event.clone());
        Ok(())
    }
}

/// Turns a decision to start an agent into a program, as an agent runtime does.
struct Brains;

impl Part for Brains {
    fn id(&self) -> &'static str {
        "brains"
    }
    fn carry_out(&mut self, effect: &Effect, world: &World) -> Outcome {
        let launch = match effect {
            Effect::StartAgent {
                group,
                role,
                issue,
                folder,
            } => world.launch(group, *role, issue, folder.as_ref()),
            Effect::Investigate { group } => world.launch_investigator(group),
            _ => return Outcome::NotMine,
        };
        let Some(launch) = launch else {
            return Outcome::Failed("unknown issue".into());
        };
        let program = Program {
            argv: vec!["agent".into(), launch.brief.task.clone()],
            env: vec![],
        };
        Outcome::Done {
            events: vec![],
            effects: vec![Effect::Run {
                launch: Box::new(launch),
                runtime: "brains".into(),
                program,
                before: None,
            }],
        }
    }
}

/// Runs programs, as a terminal runner does, and reports what it was told to.
#[derive(Default)]
struct Terms {
    polls: Vec<Polled>,
}

impl Part for Terms {
    fn id(&self) -> &'static str {
        "terms"
    }
    fn poll(&mut self, _world: &World, _now_ms: u64) -> Polled {
        if self.polls.is_empty() {
            Polled::default()
        } else {
            self.polls.remove(0)
        }
    }
    fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
        let Effect::Run { launch, .. } = effect else {
            return Outcome::NotMine;
        };
        Outcome::Done {
            events: vec![Event::AgentStarted {
                agent: launch.agent.clone(),
                group: launch.group.clone(),
                role: launch.brief.role,
                issue: launch.brief.issue.clone(),
                machine: launch.machine.clone(),
                folder: launch.folder.clone(),
                secret_digest: None,
                runtime: None,
                model: None,
            }],
            effects: vec![],
        }
    }
}

fn setup(refining_at_once: usize) -> Vec<Event> {
    vec![
        Event::ProjectAdded {
            project: ProjectId::new("ankh"),
            tracker: "fake".into(),
            name: "city/ankh".into(),
            code_host: None,
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("f1"),
            project: ProjectId::new("ankh"),
            machine: MachineId::new("morpork"),
            path: "/srv/ankh/f1".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![ProjectId::new("ankh")],
            areas: Vec::new(),
            milestones: vec![],
            refining_at_once,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]
}

fn synced(key: &str) -> Event {
    Event::IssueSynced {
        issue: IssueId::new("ankh", key),
        title: format!("Mend clock {key}"),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: key.parse().unwrap_or(0),
        closed: false,
    }
}

fn host(parts: Vec<Box<dyn Part>>, journal: Shared) -> Host {
    let mut h = Host::new(World::default(), parts, Box::new(journal));
    for e in setup(2) {
        h.handle(0, e).expect("setup");
    }
    h
}

#[test]
fn each_event_is_applied_at_the_time_it_is_logged_with() {
    let mut h = host(Vec::new(), Shared::default());
    let issue = IssueId::new("ankh", "1");
    h.handle(5_000, synced("1")).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(5_000));
    // The same again changes nothing, so the time stays.
    h.handle(7_000, synced("1")).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(5_000));
    let renamed = Event::IssueSynced {
        issue: issue.clone(),
        title: "Mend the clock tower".into(),
        milestone: Some(MilestoneId::new("m1")),
        depends_on: vec![],
        order: 1,
        closed: false,
    };
    h.handle(9_000, renamed).expect("handle");
    assert_eq!(h.world().updated.at(&issue), Some(9_000));
}

/// Gives the web addresses of every project, as a tracker or code-host family
/// does: `issues` from one part, `code` from another.
struct Places {
    issues: Option<&'static str>,
    code: Option<&'static str>,
}

impl Part for Places {
    fn id(&self) -> &'static str {
        "places"
    }

    fn addresses(&self, world: &World) -> BTreeMap<ProjectId, crate::model::Addresses> {
        world
            .projects
            .keys()
            .map(|id| {
                let at = |a: Option<&str>| a.map(|a| format!("{a}/{id}"));
                (
                    id.clone(),
                    crate::model::Addresses {
                        issues: at(self.issues),
                        code: at(self.code),
                    },
                )
            })
            .collect()
    }
}

#[test]
fn each_round_the_parts_addresses_are_put_in_the_world_and_the_first_part_wins() {
    let parts: Vec<Box<dyn Part>> = vec![
        Box::new(Places {
            issues: Some("https://tracker.example.com"),
            code: None,
        }),
        Box::new(Places {
            issues: Some("https://other.example.com"),
            code: Some("https://git.example.com"),
        }),
    ];
    let mut h = host(parts, Shared::default());
    let ankh = ProjectId::new("ankh");
    // The project came from the configuration after the host started: the next
    // round has it.
    assert!(!h.world().addresses.contains_key(&ankh));
    h.tick(0).expect("tick");
    assert_eq!(
        h.world().addresses[&ankh],
        crate::model::Addresses {
            issues: Some("https://tracker.example.com/ankh".into()),
            code: Some("https://git.example.com/ankh".into()),
        }
    );
}

#[test]
fn a_synced_issue_starts_a_refiner_through_the_parts_and_logs_it() {
    let journal = Shared::default();
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        journal.clone(),
    );
    h.handle(1, synced("1")).expect("handle");
    let agent = h
        .world()
        .agents
        .get(&AgentId::new("refiner-1"))
        .expect("refiner-1");
    assert_eq!(agent.issue, Some(IssueId::new("ankh", "1")));
    assert_eq!(agent.machine, MachineId::new("morpork"));
    assert!(h.problems().is_empty(), "{:?}", h.problems());
    let logged = journal.0.lock().expect("journal").clone();
    assert_eq!(logged.len(), 5);
    assert!(matches!(logged[4], Event::AgentStarted { .. }));
}

#[test]
fn a_tick_starts_one_investigator_for_a_group_short_of_work_and_logs_its_start_and_end() {
    let journal = Shared::default();
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        journal.clone(),
    );
    h.tick(1_000).expect("tick");
    h.tick(2_000).expect("tick");
    let me = AgentId::new("investigator-1");
    let logged = journal.0.lock().expect("journal").clone();
    let since_setup = &logged[3..];
    assert_eq!(
        since_setup,
        [
            Event::InvestigationStarted {
                group: GroupId::new("watch"),
                agent: me.clone(),
                at_ms: 1_000,
            },
            Event::AgentStarted {
                agent: me.clone(),
                group: GroupId::new("watch"),
                role: Role::Investigator,
                issue: None,
                machine: MachineId::new("morpork"),
                folder: None,
                secret_digest: None,
                runtime: None,
                model: None,
            },
        ],
        "one start, the second tick none"
    );
    // Its word ends the investigation at once, stamped with the host's clock.
    h.handle(
        3_000,
        Event::Investigated {
            group: GroupId::new("watch"),
            agent: me,
        },
    )
    .expect("handle");
    let logged = journal.0.lock().expect("journal").clone();
    assert_eq!(
        logged.last(),
        Some(&Event::InvestigationEnded {
            group: GroupId::new("watch"),
            at_ms: 3_000,
            filed: 0,
        })
    );
}

#[test]
fn agents_are_numbered_by_role_in_the_order_they_start() {
    let mut h = host(
        vec![Box::new(Brains), Box::new(Terms::default())],
        Shared::default(),
    );
    h.handle(1, synced("1")).expect("one");
    h.handle(2, synced("2")).expect("two");
    let ids: Vec<&str> = h.world().agents.keys().map(|a| a.0.as_str()).collect();
    assert_eq!(ids, ["refiner-1", "refiner-2"]);
}

#[test]
fn an_effect_no_part_takes_is_a_problem() {
    let mut h = host(vec![], Shared::default());
    h.handle(1, synced("1")).expect("handle");
    assert_eq!(
        h.problems()
            .get("start a refiner on ankh#1")
            .map(String::as_str),
        Some("no part in this build does this")
    );
}

#[test]
fn issue_text_no_part_reads_is_refused_with_the_reason() {
    let mut h = host(vec![Box::new(Terms::default())], Shared::default());
    h.handle(1, synced("1")).expect("handle");
    assert_eq!(
        h.issue_text(&IssueId::new("ankh", "1")),
        Err("no part in this build reads issue text".into())
    );
}

#[test]
fn a_failed_poll_is_a_problem_until_a_poll_succeeds() {
    let terms = Terms {
        polls: vec![
            Polled {
                events: vec![],
                problem: Some("socket refused".into()),
            },
            Polled {
                events: vec![Event::AgentChanged {
                    agent: AgentId::new("nobody"),
                    state: AgentState::Idle,
                }],
                problem: None,
            },
        ],
    };
    let mut h = host(vec![Box::new(terms)], Shared::default());
    assert_eq!(h.tick(1), Ok(0));
    assert_eq!(
        h.problems().get("terms").map(String::as_str),
        Some("socket refused")
    );
    assert_eq!(h.tick(2), Ok(1));
    assert!(h.problems().get("terms").is_none());
}

#[test]
fn a_journal_that_cannot_write_stops_the_host() {
    let mut h = Host::new(
        World::default(),
        vec![],
        Box::new(Shared(Arc::default(), true)),
    );
    assert_eq!(h.handle(0, synced("1")), Err("disk full".into()));
    assert_eq!(
        h.world().applied,
        0,
        "an event that was not logged is not applied"
    );
}

#[test]
fn a_part_that_keeps_asking_for_more_is_stopped() {
    struct Echo;
    impl Part for Echo {
        fn id(&self) -> &'static str {
            "echo"
        }
        fn carry_out(&mut self, effect: &Effect, _world: &World) -> Outcome {
            Outcome::Done {
                events: vec![],
                effects: vec![effect.clone()],
            }
        }
    }
    let mut h = Host::new(
        World::default(),
        vec![Box::new(Echo)],
        Box::new(Memory::default()),
    );
    let mut fails = Vec::new();
    for e in setup(1) {
        fails.push(h.handle(0, e));
    }
    fails.push(h.handle(0, synced("1")));
    assert!(
        fails
            .last()
            .expect("result")
            .as_ref()
            .is_err_and(|e| e.contains("steps"))
    );
}

#[test]
fn an_author_runs_in_its_folder_and_a_refiner_in_any_folder_of_the_project() {
    let mut w = World::default();
    for e in setup(1) {
        w.apply(&e);
    }
    w.apply(&synced("1"));
    w.apply(&Event::PlanWritten {
        issue: IssueId::new("ankh", "1"),
        plan: Plan {
            repos: Vec::new(),
            settings: Default::default(),
            checks: vec![crate::model::Check {
                done_when: "the clock strikes thirteen".into(),
                proven_by: "listening at midnight".into(),
            }],
        },
    });
    let group = GroupId::new("watch");
    let issue = IssueId::new("ankh", "1");
    let author = w
        .launch(&group, Role::Author, &issue, Some(&FolderId::new("f1")))
        .expect("author");
    assert_eq!(author.agent, AgentId::new("author-1"));
    assert_eq!(author.folder, Some(FolderId::new("f1")));
    assert_eq!(
        author.brief.folder.as_deref(),
        Some(std::path::Path::new("/srv/ankh/f1"))
    );
    assert!(author.brief.task.contains("the clock strikes thirteen"));
    assert!(author.brief.task.contains("temet-work ready ankh#1"));
    let refiner = w
        .launch(&group, Role::Refiner, &issue, None)
        .expect("refiner");
    assert_eq!(refiner.folder, None, "a refiner holds no folder");
    assert_eq!(
        refiner.brief.folder.as_deref(),
        Some(std::path::Path::new("/srv/ankh/f1"))
    );
    assert!(refiner.brief.task.contains("issue 1 of city/ankh"));
    assert!(
        w.launch(&group, Role::Refiner, &IssueId::new("ankh", "9"), None)
            .is_none()
    );
}
