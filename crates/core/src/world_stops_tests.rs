//! Stops that failed or were lost to a restart (#208): `World::missing` asks
//! again to stop an agent whose step is over for good, and no author is placed
//! in a folder while an agent that held it runs.

use super::*;
use crate::command::Command;
use crate::event::{Event, Verdict};
use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, ProjectId};
use crate::model::{AgentState, Plan, Role, Stage};
use crate::settings::{Scope, Value};

/// A live world and every event applied to it, so a test can replay the log
/// as the daemon does at start, where effects are dropped.
#[derive(Default)]
struct Logged {
    w: World,
    log: Vec<Event>,
}

impl Logged {
    fn apply(&mut self, event: Event) -> Vec<Effect> {
        let effects = self.w.apply(&event);
        self.log.push(event);
        effects
    }

    fn apply_all(&mut self, events: impl IntoIterator<Item = Event>) -> Vec<Effect> {
        events.into_iter().flat_map(|e| self.apply(e)).collect()
    }

    /// Runs `command` as `by` and applies what it becomes.
    fn run(&mut self, by: &str, command: &Command) -> Vec<Effect> {
        let events = self
            .w
            .events(Some(&AgentId::new(by)), command)
            .unwrap_or_else(|e| panic!("{by} is refused: {e}"));
        self.apply_all(events)
    }

    /// The world the log replays to.
    fn replayed(&self) -> World {
        let (w, _) = World::replay(&self.log);
        assert_eq!(w, self.w, "the replay gives the live state");
        w
    }
}

fn pid(id: &str) -> ProjectId {
    ProjectId::new(id)
}

fn watch(key: &str) -> IssueId {
    IssueId::new("watch", key)
}

fn agent(id: &str) -> AgentId {
    AgentId::new(id)
}

fn stop(id: &str) -> Effect {
    Effect::StopAgent { agent: agent(id) }
}

fn ended(id: &str) -> Event {
    Event::AgentChanged {
        agent: agent(id),
        state: AgentState::Ended,
        lost: false,
    }
}

fn started(id: &str, role: Role, issue: Option<IssueId>, folder: Option<&str>) -> Event {
    Event::AgentStarted {
        agent: agent(id),
        group: GroupId::new("watch"),
        role,
        issue,
        machine: MachineId::new("morpork"),
        folder: folder.map(FolderId::new),
        secret_digest: None,
        runtime: None,
        model: None,
    }
}

fn synced(issue: IssueId, closed: bool) -> Event {
    Event::IssueSynced {
        title: format!("issue {issue}"),
        issue,
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed,
    }
}

fn plan_of(issue: IssueId, repos: &[&str]) -> Event {
    Event::PlanWritten {
        issue,
        plan: Plan {
            repos: repos.iter().map(|r| pid(r)).collect(),
            ..Plan::default()
        },
    }
}

fn author_of(issue: IssueId, folder: &str) -> Effect {
    Effect::StartAgent {
        group: GroupId::new("watch"),
        role: Role::Author,
        issue,
        folder: Some(FolderId::new(folder)),
    }
}

fn starts_an_author(effects: &[Effect]) -> bool {
    effects.iter().any(|e| {
        matches!(
            e,
            Effect::StartAgent {
                role: Role::Author,
                ..
            }
        )
    })
}

fn holder(w: &World, folder: &str) -> Option<IssueId> {
    w.folders[&FolderId::new(folder)].holder.clone()
}

/// The project watch (ankh-morpork/watch) with the work folders `folders`,
/// taken by the group watch, which refines two issues at once and has one
/// review round before an arbiter rules.
fn watch_project(folders: &[&str]) -> Logged {
    let mut l = Logged::default();
    l.apply(Event::ProjectAdded {
        project: pid("watch"),
        tracker: "fake".into(),
        name: "ankh-morpork/watch".into(),
        code_host: Some("fake".into()),
        repo: None,
    });
    l.apply_all(folders.iter().map(|f| Event::FolderAdded {
        folder: FolderId::new(*f),
        project: pid("watch"),
        machine: MachineId::new("morpork"),
        path: format!("/srv/watch/{f}"),
    }));
    l.apply_all([
        Event::SettingSet {
            scope: Scope::Project(pid("watch")),
            role: None,
            key: "review_rounds".into(),
            value: Value::Number(1),
            by: "person".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: vec![pid("watch")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 2,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]);
    l
}

/// watch#12 refined by refiner-1 and worked on by author-1 in w1, with
/// watch#13 ready behind it and waiting for w1, its project's only folder.
fn twelve_working() -> Logged {
    let mut l = watch_project(&["w1"]);
    l.apply(synced(watch("12"), false));
    l.apply(started("refiner-1", Role::Refiner, Some(watch("12")), None));
    let effects = l.apply(plan_of(watch("12"), &[]));
    assert!(
        effects.contains(&author_of(watch("12"), "w1")),
        "{effects:?}"
    );
    l.apply(ended("refiner-1"));
    l.apply(started(
        "author-1",
        Role::Author,
        Some(watch("12")),
        Some("w1"),
    ));
    l.apply(synced(watch("13"), false));
    let effects = l.apply(plan_of(watch("13"), &[]));
    assert!(!starts_an_author(&effects), "{effects:?}");
    assert_eq!(l.w.issues[&watch("13")].stage, Stage::Ready);
    l
}

/// watch#12 with pull request 57 in review by reviewer-1.
fn twelve_in_review() -> Logged {
    let mut l = twelve_working();
    l.run(
        "author-1",
        &Command::Ready {
            issue: watch("12"),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        },
    );
    l.apply(started(
        "reviewer-1",
        Role::Reviewer,
        Some(watch("12")),
        None,
    ));
    l.apply(Event::Told { issue: watch("12") });
    l
}

/// watch#12 merging after reviewer-1's block went to arbiter-1, which
/// approved; the checks passed.
fn twelve_merging_by_ruling() -> Logged {
    let mut l = twelve_in_review();
    l.run(
        "reviewer-1",
        &Command::Review {
            issue: watch("12"),
            verdict: Verdict::Block {
                points: vec!["the rota leaves the gate unwatched".into()],
            },
            failed: Vec::new(),
            open: Vec::new(),
        },
    );
    assert_eq!(
        l.w.issues[&watch("12")].stage,
        Stage::Arbitration { round: 1 }
    );
    l.apply(started("arbiter-1", Role::Arbiter, Some(watch("12")), None));
    l.run(
        "arbiter-1",
        &Command::Rule {
            issue: watch("12"),
            approve: true,
            reason: "the gate has its own guard".into(),
        },
    );
    l.apply(Event::ChecksFinished {
        issue: watch("12"),
        passed: true,
        failing: Vec::new(),
    });
    assert_eq!(l.w.issues[&watch("12")].stage, Stage::Merging);
    l
}

fn merged_12() -> Event {
    Event::Merged {
        issue: watch("12"),
        repo: None,
    }
}

#[test]
fn missing_asks_again_to_stop_the_author_reviewer_and_arbiter_once_the_issue_merged() {
    let mut l = twelve_merging_by_ruling();
    // The stops the merge asks for fail, or are lost to a restart.
    let effects = l.apply(merged_12());
    for id in ["author-1", "reviewer-1", "arbiter-1"] {
        assert!(effects.contains(&stop(id)), "{id}: {effects:?}");
    }
    let stops = vec![stop("arbiter-1"), stop("author-1"), stop("reviewer-1")];
    let then = |finish: Effect| {
        let mut all = stops.clone();
        all.push(finish);
        all
    };
    // Deploying, verifying and done: each time the log is replayed, the three
    // are asked again to stop, before what the stage needs.
    assert_eq!(
        l.replayed().missing().start,
        then(Effect::Deploy { issue: watch("12") })
    );
    l.apply(Event::Deployed { issue: watch("12") });
    assert_eq!(
        l.replayed().missing().start,
        then(Effect::Verify { issue: watch("12") })
    );
    l.apply(Event::Verified {
        issue: watch("12"),
        passed: true,
        output: String::new(),
    });
    assert_eq!(l.w.issues[&watch("12")].stage, Stage::Done);
    assert_eq!(l.replayed().missing().start, stops);
    // Once each has ended, none is asked again, and w1 goes to watch#13.
    l.apply_all(["author-1", "reviewer-1", "arbiter-1"].map(ended));
    assert_eq!(l.replayed().missing().start, [author_of(watch("13"), "w1")]);
}

#[test]
fn missing_asks_again_to_stop_the_agents_of_an_issue_closed_in_its_tracker() {
    let mut l = twelve_in_review();
    let effects = l.apply(synced(watch("12"), true));
    assert_eq!(effects[..2], [stop("author-1"), stop("reviewer-1")]);
    assert_eq!(l.w.issues[&watch("12")].stage, Stage::Done);
    assert_eq!(
        l.replayed().missing().start,
        [stop("author-1"), stop("reviewer-1")]
    );
}

#[test]
fn missing_asks_again_to_stop_a_refiner_whose_plan_is_in() {
    // No work folder, so the plan starts no author.
    let mut l = watch_project(&[]);
    l.apply(synced(watch("12"), false));
    l.apply(started("refiner-1", Role::Refiner, Some(watch("12")), None));
    // While refining it is still needed.
    assert_eq!(l.replayed().missing().start, []);
    assert_eq!(l.apply(plan_of(watch("12"), &[])), [stop("refiner-1")]);
    assert_eq!(l.w.issues[&watch("12")].stage, Stage::Ready);
    assert_eq!(l.replayed().missing().start, [stop("refiner-1")]);

    // It is asked on a paused issue too: the group stops covering it.
    l.apply(Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![pid("klatch")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 2,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    let w = l.replayed();
    assert!(w.paused(&w.issues[&watch("12")]));
    assert_eq!(w.missing().start, [stop("refiner-1")]);
}

#[test]
fn missing_asks_again_to_stop_an_investigator_whose_investigation_ended() {
    let mut l = watch_project(&["w1"]);
    l.apply(Event::InvestigationStarted {
        group: GroupId::new("watch"),
        agent: agent("investigator-1"),
        at_ms: 1_000,
    });
    l.apply(started("investigator-1", Role::Investigator, None, None));
    // While its investigation runs it is still needed.
    assert_eq!(l.replayed().missing().start, []);
    let effects = l.apply(Event::Investigated {
        group: GroupId::new("watch"),
        agent: agent("investigator-1"),
    });
    assert_eq!(effects, [stop("investigator-1")]);
    let effects = l.apply(Event::InvestigationEnded {
        group: GroupId::new("watch"),
        at_ms: 2_000,
        filed: 0,
    });
    assert_eq!(effects, [stop("investigator-1")]);
    assert_eq!(l.replayed().missing().start, [stop("investigator-1")]);
    l.apply(ended("investigator-1"));
    assert_eq!(l.replayed().missing(), Missing::default());
}

#[test]
fn missing_asks_again_to_stop_no_agent_that_is_still_needed() {
    let stops = |w: &World| -> Vec<Effect> {
        w.missing()
            .start
            .into_iter()
            .filter(|e| matches!(e, Effect::StopAgent { .. }))
            .collect()
    };
    // The author and reviewer in review.
    let mut l = twelve_in_review();
    assert_eq!(stops(&l.replayed()), []);
    // Being fixed, after a failed check.
    l.apply(Event::ChecksFinished {
        issue: watch("12"),
        passed: false,
        failing: vec!["rota".into()],
    });
    assert_eq!(l.w.issues[&watch("12")].stage, Stage::Fixing { round: 1 });
    assert_eq!(stops(&l.replayed()), []);
    // Merging, with the arbiter that approved it.
    assert_eq!(stops(&twelve_merging_by_ruling().replayed()), []);
    // A refiner while its issue is refining, and the investigator of a running
    // investigation.
    let mut l = watch_project(&["w1"]);
    l.apply(synced(watch("14"), false));
    l.apply(started("refiner-1", Role::Refiner, Some(watch("14")), None));
    l.apply(Event::InvestigationStarted {
        group: GroupId::new("watch"),
        agent: agent("investigator-1"),
        at_ms: 1_000,
    });
    l.apply(started("investigator-1", Role::Investigator, None, None));
    assert_eq!(stops(&l.replayed()), []);
}

#[test]
fn merged_with_author_running_holds_its_folder() {
    let mut l = twelve_in_review();
    l.run(
        "reviewer-1",
        &Command::Review {
            issue: watch("12"),
            verdict: Verdict::Approve,
            failed: Vec::new(),
            open: Vec::new(),
        },
    );
    l.apply(Event::ChecksFinished {
        issue: watch("12"),
        passed: true,
        failing: Vec::new(),
    });
    // The merge asks for both stops; neither has worked, so both still run.
    let effects = l.apply(merged_12());
    assert!(effects.contains(&stop("author-1")), "{effects:?}");
    assert!(effects.contains(&stop("reviewer-1")), "{effects:?}");
    assert!(!starts_an_author(&effects), "{effects:?}");
    assert_eq!(holder(&l.w, "w1"), None, "watch#12 let it go");
    assert_eq!(l.w.issues[&watch("13")].stage, Stage::Ready);

    // After a restart the stops are asked for again, beside the deploy, and
    // watch#13 still gets no author in w1.
    let w = l.replayed();
    assert_eq!(
        w.missing().start,
        [
            stop("author-1"),
            stop("reviewer-1"),
            Effect::Deploy { issue: watch("12") }
        ]
    );
    assert_eq!(w.issues[&watch("13")].stage, Stage::Ready);
    assert_eq!(holder(&w, "w1"), None);

    // author-1 ends; reviewer-1, which only read w1, still runs. watch#13's
    // author starts in w1, in the live world and in the replayed one.
    assert_eq!(l.apply(ended("author-1")), [author_of(watch("13"), "w1")]);
    let w = l.replayed();
    assert_eq!(w.issues[&watch("13")].stage, Stage::Working);
    assert_eq!(holder(&w, "w1"), Some(watch("13")));
    assert_ne!(w.agents[&agent("reviewer-1")].state, AgentState::Ended);
}

#[test]
fn closed_with_author_running_holds_its_folder() {
    let mut l = twelve_working();
    // A person closes watch#12 in its tracker; author-1's stop fails.
    let effects = l.apply(synced(watch("12"), true));
    assert_eq!(
        effects,
        [
            stop("author-1"),
            Effect::ReleaseFolder {
                folder: FolderId::new("w1")
            }
        ]
    );
    let w = l.replayed();
    assert_eq!(w.issues[&watch("12")].stage, Stage::Done);
    assert_eq!(w.missing().start, [stop("author-1")]);
    assert_eq!(w.issues[&watch("13")].stage, Stage::Ready);
    // Nothing else puts an author in w1 while author-1 runs.
    let effects = l.apply(synced(watch("13"), false));
    assert!(!starts_an_author(&effects), "{effects:?}");
    assert_eq!(l.apply(ended("author-1")), [author_of(watch("13"), "w1")]);
    assert_eq!(holder(&l.replayed(), "w1"), Some(watch("13")));
}

/// An area guild of the projects ui and api, with one work folder in each and
/// the group watch taking the whole area.
fn guild() -> Logged {
    let mut l = Logged::default();
    l.apply_all([
        Event::AreaDefined {
            area: pid("guild"),
            name: "Thieves' Guild".into(),
            projects: vec![pid("ui"), pid("api")],
            tracker: "fake".into(),
            tracker_name: "GLD".into(),
        },
        Event::ProjectAdded {
            project: pid("ui"),
            tracker: String::new(),
            name: "guild/ui".into(),
            code_host: Some("fake".into()),
            repo: None,
        },
        Event::ProjectAdded {
            project: pid("api"),
            tracker: "fake".into(),
            name: "guild/api".into(),
            code_host: Some("fake".into()),
            repo: None,
        },
        Event::FolderAdded {
            folder: FolderId::new("ui-1"),
            project: pid("ui"),
            machine: MachineId::new("morpork"),
            path: "/srv/ui-1".into(),
        },
        Event::FolderAdded {
            folder: FolderId::new("api-1"),
            project: pid("api"),
            machine: MachineId::new("morpork"),
            path: "/srv/api-1".into(),
        },
        Event::GroupCreated {
            group: GroupId::new("watch"),
            name: "Night watch".into(),
            projects: Vec::new(),
            areas: vec![pid("guild")],
            milestones: Vec::new(),
            refining_at_once: 2,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]);
    l
}

#[test]
fn author_in_several_repos_holds_every_folder() {
    let gld = IssueId::new("guild", "GLD-12");
    let api = IssueId::new("api", "7");
    let mut l = guild();
    l.apply(synced(gld.clone(), false));
    let effects = l.apply(plan_of(gld.clone(), &["ui", "api"]));
    assert!(
        effects.contains(&author_of(gld.clone(), "ui-1")),
        "{effects:?}"
    );
    l.apply(started(
        "author-1",
        Role::Author,
        Some(gld.clone()),
        Some("ui-1"),
    ));
    assert_eq!(
        l.w.agents[&agent("author-1")].held,
        [FolderId::new("ui-1"), FolderId::new("api-1")]
    );
    // api#7 changes api only, and waits for api-1.
    l.apply(synced(api.clone(), false));
    assert!(!starts_an_author(&l.apply(plan_of(api.clone(), &[]))));

    // GLD-12 is closed in its tracker: both folders are let go, and author-1,
    // whose stop has not worked, still holds api-1 though it runs in ui-1.
    let effects = l.apply(synced(gld.clone(), true));
    assert!(effects.contains(&stop("author-1")), "{effects:?}");
    assert!(!starts_an_author(&effects), "{effects:?}");
    let w = l.replayed();
    assert_eq!((holder(&w, "ui-1"), holder(&w, "api-1")), (None, None));
    assert_eq!(w.issues[&api].stage, Stage::Ready);
    assert_eq!(w.missing().start, [stop("author-1")]);

    assert_eq!(
        l.apply(ended("author-1")),
        [author_of(api.clone(), "api-1")]
    );
    assert_eq!(holder(&l.replayed(), "api-1"), Some(api));
}

/// Approves watch#12's round 1, passes its checks and merges it. The stops
/// the merge asks for do not work.
fn merge_12(l: &mut Logged) {
    l.run(
        "reviewer-1",
        &Command::Review {
            issue: watch("12"),
            verdict: Verdict::Approve,
            failed: Vec::new(),
            open: Vec::new(),
        },
    );
    l.apply(Event::ChecksFinished {
        issue: watch("12"),
        passed: true,
        failing: Vec::new(),
    });
    let effects = l.apply(merged_12());
    assert!(effects.contains(&stop("author-1")), "{effects:?}");
}

fn verify_fails_12(l: &mut Logged) {
    l.apply(Event::Deployed { issue: watch("12") });
    l.apply(Event::Verified {
        issue: watch("12"),
        passed: false,
        output: "the gate stood open at midnight".into(),
    });
}

#[test]
fn missing_asks_again_to_stop_the_agents_of_an_issue_back_to_ready() {
    let mut l = twelve_in_review();
    merge_12(&mut l);
    // The verify fails while author-1 and reviewer-1 still run: watch#12 is
    // ready again, and its old agents are asked to stop after a restart.
    verify_fails_12(&mut l);
    let w = l.replayed();
    assert_eq!(w.issues[&watch("12")].stage, Stage::Ready);
    assert_eq!(w.missing().start, [stop("author-1"), stop("reviewer-1")]);
    // Neither watch#12 nor watch#13 gets w1 until author-1 ends; then watch#12,
    // first in the group's order, does.
    assert_eq!(holder(&w, "w1"), None);
    assert_eq!(l.apply(ended("author-1")), [author_of(watch("12"), "w1")]);
}

#[test]
fn missing_asks_again_to_stop_the_old_author_of_a_reopened_issue() {
    let mut l = twelve_working();
    // A person closes watch#12 while author-1 runs, the stop fails, and the
    // person reopens it: the group refines it again.
    l.apply(synced(watch("12"), true));
    l.apply(synced(watch("12"), false));
    let w = l.replayed();
    assert_eq!(w.issues[&watch("12")].stage, Stage::Refining);
    assert!(
        w.missing().start.starts_with(&[stop("author-1")]),
        "{:?}",
        w.missing()
    );
    assert_eq!(holder(&w, "w1"), None);
}

#[test]
fn missing_asks_again_to_stop_an_author_whose_issue_holds_another_folder() {
    let mut l = watch_project(&["w1", "w2"]);
    l.apply(synced(watch("12"), false));
    l.apply(started("refiner-1", Role::Refiner, Some(watch("12")), None));
    l.apply(plan_of(watch("12"), &[]));
    l.apply(ended("refiner-1"));
    l.apply(started(
        "author-1",
        Role::Author,
        Some(watch("12")),
        Some("w1"),
    ));
    l.run(
        "author-1",
        &Command::Ready {
            issue: watch("12"),
            pull: Some("57".into()),
            pulls: Default::default(),
            replies: Vec::new(),
        },
    );
    l.apply(started(
        "reviewer-1",
        Role::Reviewer,
        Some(watch("12")),
        None,
    ));
    l.apply(Event::Told { issue: watch("12") });
    merge_12(&mut l);
    l.apply(ended("reviewer-1"));
    // After the failed verify watch#12 takes w2, since author-1 still holds w1,
    // and author-2 works there.
    verify_fails_12(&mut l);
    assert_eq!(l.w.issues[&watch("12")].folder, Some(FolderId::new("w2")));
    l.apply(started(
        "author-2",
        Role::Author,
        Some(watch("12")),
        Some("w2"),
    ));
    let w = l.replayed();
    assert_eq!(w.missing().start, [stop("author-1")]);
    // Its issue waits on author-2 alone, so author-1 is not reminded.
    assert!(!w.waiting_on(&w.agents[&agent("author-1")]));
    assert!(w.waiting_on(&w.agents[&agent("author-2")]));
    l.apply(ended("author-1"));
    assert_eq!(l.replayed().missing(), Missing::default());
}

#[test]
fn missing_asks_again_to_stop_the_old_author_of_a_reopened_issue_left_in_new() {
    let stops = |w: &World| -> Vec<Effect> {
        w.missing()
            .start
            .into_iter()
            .filter(|e| matches!(e, Effect::StopAgent { .. }))
            .collect()
    };
    // Both of the group's refining slots are taken when a person closes
    // watch#12, author-1's stop fails, and the person reopens it.
    let mut l = twelve_working();
    l.apply_all([synced(watch("14"), false), synced(watch("15"), false)]);
    l.apply(synced(watch("12"), true));
    l.apply(synced(watch("12"), false));
    let w = l.replayed();
    assert_eq!(w.issues[&watch("12")].stage, Stage::New, "waits for a slot");
    assert_eq!(stops(&w), [stop("author-1")]);
    assert_eq!(holder(&w, "w1"), None);
    assert_eq!(w.issues[&watch("13")].stage, Stage::Ready);
    // Once author-1 ends, watch#13 takes w1.
    let mut ends = Logged {
        w: w.clone(),
        log: l.log.clone(),
    };
    assert_eq!(
        ends.apply(ended("author-1")),
        [author_of(watch("13"), "w1")]
    );

    // No group covers it any longer: it stays new, and the stop is still asked.
    l.apply(Event::GroupCreated {
        group: GroupId::new("watch"),
        name: "Night watch".into(),
        projects: vec![pid("klatch")],
        areas: Vec::new(),
        milestones: Vec::new(),
        refining_at_once: 2,
        model: None,
        queue_low: None,
        investigate_every: None,
    });
    let w = l.replayed();
    assert_eq!(w.issues[&watch("12")].stage, Stage::New);
    assert_eq!(stops(&w), [stop("author-1")]);
}
