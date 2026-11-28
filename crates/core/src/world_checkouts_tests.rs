//! Readers' own checkouts: each reader starts in a checkout of its own, never in
//! a work folder, and its end asks for the checkout's removal.

use super::*;
use crate::model::{AgentCheckout, AgentState, Checkout};

/// The log id the checkouts of these worlds are made under.
const LOG: &str = "l1";

/// watch#12 being worked on by author-1, who holds watch's only work folder, w1.
fn working() -> World {
    let mut w = watch_12_working();
    w.apply(&Event::LogStarted { id: LOG.into() });
    w
}

/// watch#12 in review round 1 with pull request 57, before the code host has
/// reported its branch.
fn ready_57() -> (World, Vec<Effect>) {
    let mut w = working();
    let ready = ready_12(&w, "author-1", Some("57")).expect("ready 57");
    let effects = apply_all(&mut w, &ready);
    (w, effects)
}

/// The same, once the code host has reported 57's branch.
fn reviewing_57() -> World {
    let (mut w, _) = ready_57();
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "57".into(),
        branch: "watch-12-night-shift".into(),
    });
    w
}

/// The checkout `name` of `project`, for `agent`, added from `from` on morpork
/// at `branch`.
fn checkout(name: &str, agent: &str, from: &str, branch: Option<&str>) -> Checkout {
    let project = name.rsplit_once('-').map_or(name, |(p, _)| p);
    let parent = from.rsplit_once('/').map_or("", |(p, _)| p);
    Checkout {
        name: name.into(),
        agent: AgentId::new(agent),
        project: pid(project),
        machine: MachineId::new("morpork"),
        path: format!("{parent}/temet-work-checkouts/{LOG}/{name}"),
        from: from.into(),
        branch: branch.map(String::from),
        kept: None,
    }
}

/// `agent` starting with `checkouts`, as the runner reports it.
fn started_in(agent: &str, role: Role, issue: Option<IssueId>, checkouts: &[Checkout]) -> Event {
    Event::AgentStarted {
        agent: AgentId::new(agent),
        group: GroupId::new("watch"),
        role,
        issue,
        machine: MachineId::new("morpork"),
        folder: None,
        secret_digest: None,
        runtime: None,
        model: None,
        checkouts: checkouts
            .iter()
            .map(|c| AgentCheckout {
                project: c.project.clone(),
                path: c.path.clone(),
            })
            .collect(),
    }
}

fn added(c: &Checkout) -> Event {
    Event::CheckoutAdded {
        checkout: c.clone(),
    }
}

fn removal(c: &Checkout) -> Effect {
    Effect::RemoveCheckout {
        checkout: c.clone(),
    }
}

fn review_1() -> Effect {
    Effect::StartReview {
        issue: watch_12(),
        round: 1,
    }
}

#[test]
fn reviewer_1_starts_in_a_checkout_of_its_own_while_author_1_holds_the_only_folder() {
    let mut w = reviewing_57();
    assert_eq!(w.folders[&FolderId::new("w1")].holder, Some(watch_12()));
    let launch = w
        .launch(&GroupId::new("watch"), Role::Reviewer, &watch_12(), None)
        .expect("launch");
    let own = checkout(
        "watch-1",
        "reviewer-1",
        "/srv/watch/w1",
        Some("watch-12-night-shift"),
    );
    assert_eq!(launch.agent, AgentId::new("reviewer-1"));
    assert_eq!(launch.folder, None, "it holds no work folder");
    assert_eq!(launch.checkouts, vec![own.clone()]);
    assert_eq!(
        launch.brief.folder,
        Some("/srv/watch/temet-work-checkouts/l1/watch-1".into())
    );
    assert!(
        launch
            .brief
            .task
            .contains("Your own checkout of watch (ankh-morpork/watch) is at /srv/watch/temet-work-checkouts/l1/watch-1, at the head of branch watch-12-night-shift"),
        "{}",
        launch.brief.task
    );
    // Its program waits for the checkout: a git worktree of watch at the pull
    // request's head, added from author-1's folder's repository.
    assert_eq!(
        w.checkout_to_make(&launch, &review_1()),
        Some(Effect::AddCheckout {
            checkout: own.clone(),
            then: Box::new(review_1()),
        })
    );
    w.apply(&added(&own));
    let again = w
        .launch(&GroupId::new("watch"), Role::Reviewer, &watch_12(), None)
        .expect("launch");
    assert_eq!(again.checkouts, vec![own.clone()], "the same checkout");
    assert_eq!(w.checkout_to_make(&again, &review_1()), None);
    w.apply(&started_in(
        "reviewer-1",
        Role::Reviewer,
        Some(watch_12()),
        std::slice::from_ref(&own),
    ));
    assert_eq!(
        w.agents[&AgentId::new("reviewer-1")].checkouts,
        [AgentCheckout {
            project: pid("watch"),
            path: own.path.clone(),
        }]
    );
    // Its end asks for the checkout's removal.
    assert_eq!(w.apply(&ended("reviewer-1")), vec![removal(&own)]);
}

#[test]
fn a_reviewer_waits_for_the_branch_of_its_pull_request_before_it_starts() {
    let (w, effects) = ready_57();
    assert_eq!(w.issues[&watch_12()].stage, Stage::InReview { round: 1 });
    assert!(!effects.contains(&review_1()), "{effects:?}");
    assert!(!w.missing().start.contains(&review_1()));
    assert_eq!(w.machine_waits(), vec![], "not a wait for a machine");
    let w = reviewing_57();
    assert!(w.missing().start.contains(&review_1()));
    let launch = w
        .launch(&GroupId::new("watch"), Role::Reviewer, &watch_12(), None)
        .expect("launch");
    let branches: Vec<Option<&str>> = launch
        .checkouts
        .iter()
        .map(|c| c.branch.as_deref())
        .collect();
    assert_eq!(branches, [Some("watch-12-night-shift")]);
    // An arbiter waits the same way, here for the branch of a new pull request.
    let mut w = reviewing_57();
    w.apply(&Event::PullRequestReady {
        issue: watch_12(),
        pull: None,
        pulls: [(pid("watch"), "58".to_string())].into(),
        replies: Vec::new(),
    });
    let arbiter = Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Arbiter,
        issue: watch_12(),
        folder: None,
    };
    assert!(w.waits_for_branch(&arbiter));
    w.apply(&Event::PullBranch {
        issue: watch_12(),
        repo: pid("watch"),
        number: "58".into(),
        branch: "watch-12-rota".into(),
    });
    assert!(!w.waits_for_branch(&arbiter));
}

/// A world in which each reader role on watch has what it needs to start, with
/// author-1 holding w1 and, when `free` is set, a second folder w2 free.
fn readers(free: bool) -> World {
    let mut w = reviewing_57();
    if free {
        w.apply(&Event::FolderAdded {
            folder: FolderId::new("w2"),
            project: pid("watch"),
            machine: MachineId::new("morpork"),
            path: "/srv/watch/w2".into(),
        });
    }
    w.apply(&Event::IssueSynced {
        issue: IssueId::new("watch", "13"),
        title: "A bell for the night shift".into(),
        milestone: None,
        depends_on: vec![],
        order: 2,
        closed: false,
    });
    w.apply(&Event::InvestigationStarted {
        group: GroupId::new("watch"),
        agent: AgentId::new("investigator-1"),
        at_ms: 0,
    });
    w
}

/// The launch of `role` in `w`: on watch#13 for a refiner, the investigation for
/// an investigator, watch#12 for the others.
fn launch_of(w: &World, role: Role) -> Launch {
    let group = GroupId::new("watch");
    match role {
        Role::Investigator => w.launch_investigator(&group),
        Role::Refiner => w.launch(&group, role, &IssueId::new("watch", "13"), None),
        _ => w.launch(&group, role, &watch_12(), None),
    }
    .expect("launch")
}

#[test]
fn every_reader_runs_in_its_own_checkout_with_the_only_folder_held_or_one_free() {
    for free in [false, true] {
        let w = readers(free);
        assert_eq!(
            w.issues[&IssueId::new("watch", "13")].stage,
            Stage::Refining
        );
        for (role, branch) in [
            (Role::Refiner, None),
            (Role::Reviewer, Some("watch-12-night-shift")),
            (Role::Arbiter, Some("watch-12-night-shift")),
            (Role::Investigator, None),
        ] {
            let launch = launch_of(&w, role);
            let [own] = launch.checkouts.as_slice() else {
                panic!("{role:?}: {:?}", launch.checkouts);
            };
            assert_eq!(own.branch.as_deref(), branch, "{role:?}");
            assert_eq!(own.project, pid("watch"));
            assert_eq!(
                own.path, "/srv/watch/temet-work-checkouts/l1/watch-1",
                "{role:?}"
            );
            // The folder its launch and brief name is its checkout, never a work
            // folder, held or free.
            assert_eq!(launch.brief.folder, Some(own.path.clone().into()));
            assert!(!w.folders.values().any(|f| f.path == own.path));
            for f in w.folders.values() {
                assert!(
                    !launch.brief.task.contains(&f.path),
                    "{role:?}, free {free}: {} in:\n{}",
                    f.path,
                    launch.brief.task
                );
            }
            assert!(!launch.brief.task.contains("not yours"), "{role:?}");
        }
    }
}

#[test]
fn a_reader_with_no_work_folder_on_its_machine_runs_in_no_folder() {
    let mut w = readers(false);
    w.apply(&Event::SettingSet {
        scope: Scope::Global,
        role: Some(Role::Refiner),
        key: "machine".into(),
        value: Value::Text(LOCAL.into()),
        by: "person".into(),
    });
    // A project with no folder at all runs a reader here, which `local` allows.
    let mut bare = World::default();
    for e in [
        Event::LogStarted { id: LOG.into() },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![pid("watch")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::IssueSynced {
            issue: watch_12(),
            title: "A night shift rota".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
    ] {
        bare.apply(&e);
    }
    let launch = bare
        .launch(&GroupId::new("watch"), Role::Refiner, &watch_12(), None)
        .expect("launch");
    assert_eq!(launch.checkouts, vec![]);
    assert_eq!(launch.brief.folder, None);
    assert_eq!(launch.machine, MachineId::new(LOCAL));
    assert!(
        launch
            .brief
            .task
            .contains("watch has no work folder on your machine, so you have no checkout of it"),
        "{}",
        launch.brief.task
    );
    // Its machine has no folder of watch: it waits, and is never put in one.
    assert!(
        w.launch(
            &GroupId::new("watch"),
            Role::Refiner,
            &IssueId::new("watch", "13"),
            None
        )
        .is_none()
    );
}

#[test]
fn a_removed_checkouts_number_is_used_again_and_a_failed_start_keeps_its_checkout() {
    let mut w = readers(false);
    let first = launch_of(&w, Role::Reviewer);
    let watch_1 = first.checkouts[0].clone();
    w.apply(&added(&watch_1));
    w.apply(&started_in(
        "reviewer-1",
        Role::Reviewer,
        Some(watch_12()),
        std::slice::from_ref(&watch_1),
    ));
    // While reviewer-1's checkout exists, the arbiter gets the next number.
    let arbiter = launch_of(&w, Role::Arbiter);
    assert_eq!(arbiter.checkouts[0].name, "watch-2");
    assert_eq!(
        arbiter.checkouts[0].path,
        "/srv/watch/temet-work-checkouts/l1/watch-2"
    );
    // Made, but its program did not start: its next try is given the same one.
    w.apply(&added(&arbiter.checkouts[0]));
    let again = launch_of(&w, Role::Arbiter);
    assert_eq!(again.checkouts, arbiter.checkouts);
    // Once watch-1 is removed, its number is free for the next reader.
    w.apply(&ended("reviewer-1"));
    w.apply(&Event::CheckoutRemoved {
        name: "watch-1".into(),
    });
    let refiner = launch_of(&w, Role::Refiner);
    assert_eq!(refiner.checkouts[0].name, "watch-1");
    assert_eq!(refiner.checkouts[0].path, watch_1.path);
    // A kept checkout keeps its number.
    w.apply(&added(&refiner.checkouts[0]));
    w.apply(&Event::CheckoutKept {
        name: "watch-1".into(),
        changes: vec!["2 files changed or not ignored".into()],
    });
    let investigator = launch_of(&w, Role::Investigator);
    assert_eq!(investigator.checkouts[0].name, "watch-3");
}

#[test]
fn a_readers_end_asks_for_the_removal_of_its_checkout_however_it_ended() {
    for role in [
        Role::Refiner,
        Role::Reviewer,
        Role::Arbiter,
        Role::Investigator,
    ] {
        let mut w = readers(false);
        let launch = launch_of(&w, role);
        let own = launch.checkouts[0].clone();
        let agent = launch.agent.0.clone();
        w.apply(&added(&own));
        w.apply(&started_in(
            &agent,
            role,
            launch.brief.issue.clone(),
            std::slice::from_ref(&own),
        ));
        // Gone, lost, or ended after the loop stopped it, as after its step was
        // done or its issue closed: each reaches the loop as an end.
        for how in ["gone", "lost", "stopped"] {
            let mut w = w.clone();
            if how == "stopped" {
                let stops = w.apply(&Event::IssueSynced {
                    issue: watch_12(),
                    title: "A night shift rota".into(),
                    milestone: None,
                    depends_on: vec![],
                    order: 1,
                    closed: true,
                });
                if role == Role::Reviewer || role == Role::Arbiter {
                    assert!(
                        stops.contains(&Effect::StopAgent {
                            agent: AgentId::new(&agent)
                        }),
                        "{stops:?}"
                    );
                }
            }
            let end = Event::AgentChanged {
                agent: AgentId::new(&agent),
                state: AgentState::Ended,
                lost: how == "lost",
            };
            assert_eq!(w.apply(&end), vec![removal(&own)], "{role:?}, {how}");
            // An update after the end asks for nothing more; a round asks again
            // while it is not removed.
            assert_eq!(w.apply(&end), vec![], "{role:?}, {how}");
            assert!(w.missing().start.contains(&removal(&own)));
            w.apply(&Event::CheckoutRemoved {
                name: own.name.clone(),
            });
            assert!(!w.missing().start.contains(&removal(&own)));
        }
    }
}

#[test]
fn a_replay_with_an_ended_reviewers_checkout_still_recorded_asks_for_its_removal() {
    let mine = checkout(
        "watch-1",
        "reviewer-1",
        "/srv/watch/w1",
        Some("watch-12-night-shift"),
    );
    let never = checkout("watch-2", "arbiter-1", "/srv/watch/w1", None);
    let running = checkout("watch-3", "refiner-1", "/srv/watch/w1", None);
    let gone = checkout("watch-4", "refiner-2", "/srv/watch/w1", None);
    let kept = checkout("watch-5", "refiner-3", "/srv/watch/w1", None);
    let mut events = vec![
        Event::LogStarted { id: LOG.into() },
        added(&mine),
        started_in(
            "reviewer-1",
            Role::Reviewer,
            Some(watch_12()),
            std::slice::from_ref(&mine),
        ),
        ended("reviewer-1"),
        // Made before a crash, and its agent never started.
        added(&never),
        added(&running),
        started_in(
            "refiner-1",
            Role::Refiner,
            Some(watch_12()),
            std::slice::from_ref(&running),
        ),
        added(&gone),
        started_in(
            "refiner-2",
            Role::Refiner,
            Some(watch_12()),
            std::slice::from_ref(&gone),
        ),
        ended("refiner-2"),
        Event::CheckoutRemoved {
            name: "watch-4".into(),
        },
        added(&kept),
        started_in(
            "refiner-3",
            Role::Refiner,
            Some(watch_12()),
            std::slice::from_ref(&kept),
        ),
        ended("refiner-3"),
        Event::CheckoutKept {
            name: "watch-5".into(),
            changes: vec!["1 commit no remote-tracking branch holds".into()],
        },
    ];
    events.splice(
        0..0,
        [Event::ProjectAdded {
            project: pid("watch"),
            tracker: "fake".into(),
            name: "ankh-morpork/watch".into(),
            code_host: None,
            repo: None,
        }],
    );
    let (w, _) = World::replay(&events);
    let kept_now = Checkout {
        kept: Some(vec!["1 commit no remote-tracking branch holds".into()]),
        ..kept
    };
    assert_eq!(
        w.sweep(),
        vec![removal(&mine), removal(&never), removal(&kept_now)],
        "the ended reviewer's, the never started arbiter's, and the kept one, checked again"
    );
}

/// The area city over watch and clacks, both with a folder on morpork, and
/// semaphore with none, taken by the group watch; CITY-7 planned for watch and
/// clacks, its author holding both folders.
fn city() -> World {
    let mut w = World::default();
    let mut events = vec![
        Event::LogStarted { id: LOG.into() },
        Event::AreaDefined {
            area: pid("city"),
            name: "Ankh-Morpork".into(),
            projects: vec![pid("watch"), pid("clacks"), pid("semaphore")],
            tracker: "fake".into(),
            tracker_name: "CITY".into(),
        },
    ];
    for p in ["watch", "clacks", "semaphore"] {
        events.push(Event::ProjectAdded {
            project: pid(p),
            tracker: String::new(),
            name: format!("ankh-morpork/{p}"),
            code_host: Some("fake".into()),
            repo: None,
        });
    }
    for (f, p) in [("w1", "watch"), ("c1", "clacks")] {
        events.push(Event::FolderAdded {
            folder: FolderId::new(f),
            project: pid(p),
            machine: MachineId::new("morpork"),
            path: format!("/srv/{p}/{f}"),
        });
    }
    events.extend([
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![],
            areas: vec![pid("city")],
            milestones: vec![],
            refining_at_once: 2,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
        Event::IssueSynced {
            issue: city_7(),
            title: "Clacks towers report to the Watch".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        },
        Event::PlanWritten {
            issue: city_7(),
            plan: Plan {
                repos: vec![pid("watch"), pid("clacks")],
                ..Plan::default()
            },
        },
        Event::PullRequestReady {
            issue: city_7(),
            pull: None,
            pulls: [
                (pid("watch"), "57".to_string()),
                (pid("clacks"), "8".to_string()),
            ]
            .into(),
            replies: Vec::new(),
        },
    ]);
    for (repo, number, branch) in [
        ("watch", "57", "city-7-watch"),
        ("clacks", "8", "city-7-clacks"),
    ] {
        events.push(Event::PullBranch {
            issue: city_7(),
            repo: pid(repo),
            number: number.into(),
            branch: branch.into(),
        });
    }
    for e in &events {
        w.apply(e);
    }
    w
}

fn city_7() -> IssueId {
    IssueId::new("city", "CITY-7")
}

fn made(launch: &Launch) -> Vec<(&str, &str, &str, Option<&str>)> {
    launch
        .checkouts
        .iter()
        .map(|c| {
            (
                c.project.0.as_str(),
                c.path.as_str(),
                c.from.as_str(),
                c.branch.as_deref(),
            )
        })
        .collect()
}

#[test]
fn readers_of_an_area_issue_get_a_checkout_in_each_repository_they_read() {
    let mut w = city();
    let i = &w.issues[&city_7()];
    assert_eq!(i.folder, Some(FolderId::new("w1")));
    let group = GroupId::new("watch");
    let watch = "/srv/watch/temet-work-checkouts/l1/watch-1";
    let clacks = "/srv/clacks/temet-work-checkouts/l1/clacks-1";
    let reviewer = w
        .launch(&group, Role::Reviewer, &city_7(), None)
        .expect("reviewer");
    assert_eq!(
        made(&reviewer),
        [
            ("watch", watch, "/srv/watch/w1", Some("city-7-watch")),
            ("clacks", clacks, "/srv/clacks/c1", Some("city-7-clacks")),
        ]
    );
    assert_eq!(
        reviewer.brief.folder,
        Some(watch.into()),
        "it starts in the first"
    );
    // Once watch's pull request merged, the arbiter reads clacks's alone.
    w.issues
        .get_mut(&city_7())
        .expect("CITY-7")
        .merged
        .insert(pid("watch"));
    let arbiter = w
        .launch(&group, Role::Arbiter, &city_7(), None)
        .expect("arbiter");
    assert_eq!(
        made(&arbiter),
        [("clacks", clacks, "/srv/clacks/c1", Some("city-7-clacks"))]
    );
    // A refiner of the area's next issue reads each of its repositories at the
    // default branch; semaphore has no work folder, so it reads that one on its
    // code host.
    w.apply(&Event::IssueSynced {
        issue: IssueId::new("city", "CITY-8"),
        title: "Semaphore relays".into(),
        milestone: None,
        depends_on: vec![],
        order: 2,
        closed: false,
    });
    let refiner = w
        .launch(&group, Role::Refiner, &IssueId::new("city", "CITY-8"), None)
        .expect("refiner");
    assert_eq!(
        made(&refiner),
        [
            ("watch", watch, "/srv/watch/w1", None),
            ("clacks", clacks, "/srv/clacks/c1", None),
        ]
    );
    assert!(
        refiner
            .brief
            .task
            .contains("semaphore has no work folder on your machine, so you have no checkout of it: read it on its code host."),
        "{}",
        refiner.brief.task
    );
    // The group's investigator reads each of the group's repositories.
    w.apply(&Event::InvestigationStarted {
        group: group.clone(),
        agent: AgentId::new("investigator-1"),
        at_ms: 0,
    });
    let investigator = w.launch_investigator(&group).expect("investigator");
    assert_eq!(
        made(&investigator),
        [
            ("watch", watch, "/srv/watch/w1", None),
            ("clacks", clacks, "/srv/clacks/c1", None),
        ]
    );
}

#[test]
fn a_start_event_records_its_checkouts_and_an_older_line_reads_as_having_none() {
    let own = checkout(
        "watch-1",
        "reviewer-1",
        "/srv/watch/w1",
        Some("watch-12-night-shift"),
    );
    let started = started_in(
        "reviewer-1",
        Role::Reviewer,
        Some(watch_12()),
        std::slice::from_ref(&own),
    );
    let json = serde_json::to_string(&started).expect("json");
    assert!(
        json.contains(
            r#""checkouts":[{"project":"watch","path":"/srv/watch/temet-work-checkouts/l1/watch-1"}]"#
        ),
        "{json}"
    );
    assert_eq!(
        serde_json::from_str::<Event>(&json).expect("reads back"),
        started
    );
    for event in [
        added(&own),
        Event::CheckoutKept {
            name: "watch-1".into(),
            changes: vec!["1 commit no remote-tracking branch holds".into()],
        },
        Event::CheckoutRemoved {
            name: "watch-1".into(),
        },
    ] {
        let json = serde_json::to_string(&event).expect("json");
        assert_eq!(serde_json::from_str::<Event>(&json).expect("reads"), event);
    }
    // As a daemon wrote it before readers had checkouts.
    let old = r#"{"AgentStarted":{"agent":"reviewer-1","group":"watch","role":"Reviewer","issue":{"project":"watch","key":"12"},"machine":"morpork","folder":null}}"#;
    let mut w = working();
    w.apply(&serde_json::from_str::<Event>(old).expect("an older line reads"));
    assert_eq!(w.agents[&AgentId::new("reviewer-1")].checkouts, vec![]);
    // Its end asks for nothing to be removed.
    assert_eq!(w.apply(&ended("reviewer-1")), vec![]);
}

#[test]
fn a_blocked_or_errored_readers_problem_line_names_its_checkout() {
    let mut w = reviewing_57();
    let own = checkout(
        "watch-1",
        "reviewer-1",
        "/srv/watch/w1",
        Some("watch-12-night-shift"),
    );
    w.apply(&added(&own));
    w.apply(&started_in(
        "reviewer-1",
        Role::Reviewer,
        Some(watch_12()),
        std::slice::from_ref(&own),
    ));
    w.apply(&Event::AgentChanged {
        agent: AgentId::new("reviewer-1"),
        state: AgentState::Blocked {
            reason: "permission".into(),
        },
        lost: false,
    });
    let mut host = crate::Host::new(w, Vec::new(), Box::new(crate::host::Memory::default()));
    host.tick(0).expect("tick");
    assert_eq!(
        host.problems()["blocked: reviewer-1"],
        "reviewer-1 in /srv/watch/temet-work-checkouts/l1/watch-1 is blocked: permission; the \
         daemon cannot answer it"
    );
}
