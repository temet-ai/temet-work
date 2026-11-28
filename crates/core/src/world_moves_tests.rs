//! A person's move of an issue to another stage (#250): back to new is refined
//! again, back to ready means a new author, every other move is refused with
//! its reason, and no agent started before a move works on the issue again.

use super::*;
use crate::command::Command;
use crate::event::{Event, Verdict};
use crate::ids::{AgentId, FolderId, GroupId, IssueId, MachineId, ProjectId};
use crate::model::{AgentState, Check, CheckState, Plan, Role, Stage};
use crate::settings::{Scope, Value};
use crate::view::{StageWord, View};

/// A live world and every event applied to it, so a test can replay the log
/// as the daemon does at start.
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

    /// Runs `command` as a person and applies what it becomes.
    fn person(&mut self, command: &Command) -> Vec<Effect> {
        let events = self
            .w
            .events(None, command)
            .unwrap_or_else(|e| panic!("a person is refused: {e}"));
        self.apply_all(events)
    }

    /// Runs `command` as the agent `by` and applies what it becomes.
    fn run(&mut self, by: &str, command: &Command) -> Vec<Effect> {
        let events = self
            .w
            .events(Some(&agent(by)), command)
            .unwrap_or_else(|e| panic!("{by} is refused: {e}"));
        self.apply_all(events)
    }

    /// Why `command` from `by` (a person for none) is refused; it becomes no
    /// event.
    fn refused(&self, by: Option<&str>, command: &Command) -> String {
        match self.w.events(by.map(agent).as_ref(), command) {
            Err(why) => why,
            Ok(events) => panic!("taken as {events:?}"),
        }
    }

    /// The world the log replays to, which is the live one.
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

/// The issue every test moves.
fn twelve() -> IssueId {
    watch("12")
}

fn agent(id: &str) -> AgentId {
    AgentId::new(id)
}

fn group() -> GroupId {
    GroupId::new("watch")
}

fn stop(id: &str) -> Effect {
    Effect::StopAgent { agent: agent(id) }
}

fn release(folder: &str) -> Effect {
    Effect::ReleaseFolder {
        folder: FolderId::new(folder),
    }
}

fn ended(id: &str) -> Event {
    Event::AgentChanged {
        agent: agent(id),
        state: AgentState::Ended,
        lost: false,
    }
}

fn started(id: &str, role: Role, issue: IssueId, folder: Option<&str>) -> Event {
    Event::AgentStarted {
        agent: agent(id),
        group: group(),
        role,
        issue: Some(issue),
        machine: MachineId::new("morpork"),
        folder: folder.map(FolderId::new),
        secret_digest: None,
        runtime: None,
        model: None,
    }
}

fn synced(issue: IssueId, order: u64) -> Event {
    Event::IssueSynced {
        title: format!("issue {issue}"),
        issue,
        milestone: None,
        depends_on: vec![],
        order,
        closed: false,
    }
}

/// A plan with one check, waiting, as a refiner's plan is recorded.
fn plan() -> Plan {
    Plan {
        checks: vec![Check {
            state: Some(CheckState::Waiting),
            ..Check::new(
                "a message names the tower that sent it",
                "cargo test overhead",
            )
        }],
        ..Plan::default()
    }
}

fn plan_of(issue: IssueId) -> Event {
    Event::PlanWritten {
        issue,
        plan: plan(),
    }
}

fn refiner_of(issue: IssueId) -> Effect {
    Effect::StartAgent {
        group: group(),
        role: Role::Refiner,
        issue,
        folder: None,
    }
}

fn author_of(issue: IssueId, folder: &str) -> Effect {
    Effect::StartAgent {
        group: group(),
        role: Role::Author,
        issue,
        folder: Some(FolderId::new(folder)),
    }
}

fn starts_on(effects: &[Effect], issue: &IssueId) -> bool {
    effects
        .iter()
        .any(|e| matches!(e, Effect::StartAgent { issue: i, .. } if i == issue))
}

fn move_to(issue: IssueId, stage: StageWord) -> Command {
    Command::Move { issue, stage }
}

fn pull_ready(pulls: &[(&str, &str)], replies: &[&str]) -> Event {
    Event::PullRequestReady {
        issue: twelve(),
        pull: None,
        pulls: pulls.iter().map(|(r, n)| (pid(r), n.to_string())).collect(),
        replies: replies.iter().map(ToString::to_string).collect(),
    }
}

fn verdict(round: u8, verdict: Verdict, failed: Vec<usize>) -> Event {
    Event::ReviewFinished {
        issue: twelve(),
        round,
        verdict,
        failed: Some(failed),
        open: Some(Vec::new()),
    }
}

fn block(point: &str) -> Verdict {
    Verdict::Block {
        points: vec![point.into()],
    }
}

/// The project watch with the work folders `folders`, two review rounds before
/// an arbiter rules, and the group watch, which refines one issue at a time.
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
            value: Value::Number(2),
            by: "person".into(),
        },
        Event::GroupCreated {
            group: group(),
            name: "Night watch".into(),
            projects: vec![pid("watch")],
            areas: Vec::new(),
            milestones: Vec::new(),
            refining_at_once: 1,
            model: None,
            queue_low: None,
            investigate_every: None,
        },
    ]);
    l
}

/// watch#12 refined by refiner-1, with watch#13 at new behind it, waiting for
/// the group's one refining slot.
fn refining() -> Logged {
    let mut l = watch_project(&["w1"]);
    assert!(l.apply(synced(twelve(), 5)).contains(&refiner_of(twelve())));
    l.apply(started("refiner-1", Role::Refiner, twelve(), None));
    assert!(!starts_on(&l.apply(synced(watch("13"), 1)), &watch("13")));
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Refining);
    l
}

/// watch#13 takes the refining slot watch#12 left, so a move of watch#12 back
/// to new waits for the slot.
fn thirteen_refining(l: &mut Logged) {
    assert!(
        l.apply(synced(watch("13"), 1))
            .contains(&refiner_of(watch("13")))
    );
    l.apply(started("refiner-2", Role::Refiner, watch("13"), None));
}

/// watch#12 planned and ready, with no work folder for its author yet: the
/// project has none. watch#13 is refining.
fn ready() -> Logged {
    let mut l = watch_project(&[]);
    l.apply(synced(twelve(), 5));
    l.apply(started("refiner-1", Role::Refiner, twelve(), None));
    assert!(!starts_on(&l.apply(plan_of(twelve())), &twelve()));
    l.apply(ended("refiner-1"));
    thirteen_refining(&mut l);
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Ready);
    l
}

/// watch#12 worked on by author-1 in the first of `folders`; watch#13 is
/// refining.
fn working(folders: &[&str]) -> Logged {
    let mut l = watch_project(folders);
    l.apply(synced(twelve(), 5));
    l.apply(started("refiner-1", Role::Refiner, twelve(), None));
    assert!(
        l.apply(plan_of(twelve()))
            .contains(&author_of(twelve(), folders[0]))
    );
    l.apply(ended("refiner-1"));
    l.apply(started(
        "author-1",
        Role::Author,
        twelve(),
        Some(folders[0]),
    ));
    thirteen_refining(&mut l);
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Working);
    l
}

/// watch#12 with pull request 57 on the branch fix/12-gaspode in review round
/// 1 by reviewer-1.
fn in_review() -> Logged {
    let mut l = working(&["w1"]);
    l.apply(pull_ready(&[("watch", "57")], &[]));
    l.apply(started("reviewer-1", Role::Reviewer, twelve(), None));
    l.apply(Event::PullBranch {
        issue: twelve(),
        repo: pid("watch"),
        number: "57".into(),
        branch: "fix/12-gaspode".into(),
    });
    assert_eq!(l.w.issues[&twelve()].stage, Stage::InReview { round: 1 });
    l
}

/// watch#12 back with author-1 after round 1 blocked on its one check.
fn fixing() -> Logged {
    let mut l = in_review();
    l.apply(verdict(1, block("say which tower sent it"), vec![1]));
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Fixing { round: 1 });
    l
}

/// watch#12 with arbiter-1 after round 2 blocked too.
fn arbitration() -> Logged {
    let mut l = fixing();
    l.apply(pull_ready(&[], &["it says so now"]));
    l.apply(Event::Told { issue: twelve() });
    l.apply(verdict(2, block("still the wrong tower"), vec![1]));
    l.apply(started("arbiter-1", Role::Arbiter, twelve(), None));
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Arbitration { round: 2 });
    l
}

/// watch#12 approved with its checks passed: it is merging.
fn merging() -> Logged {
    let mut l = in_review();
    l.apply(verdict(1, Verdict::Approve, vec![]));
    l.apply(Event::ChecksFinished {
        issue: twelve(),
        passed: true,
        failing: vec![],
    });
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Merging);
    l
}

fn deploying() -> Logged {
    let mut l = merging();
    l.apply(Event::Merged {
        issue: twelve(),
        repo: Some(pid("watch")),
    });
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Deploying);
    l
}

fn verifying() -> Logged {
    let mut l = deploying();
    l.apply(Event::Deployed { issue: twelve() });
    l
}

fn done() -> Logged {
    let mut l = verifying();
    l.apply(Event::Verified {
        issue: twelve(),
        passed: true,
        output: String::new(),
    });
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Done);
    l
}

/// What a move of watch#12 back to new must leave: at new, with no group, no
/// plan, no folder, no agent running and nothing of its change; each of
/// `running` stopped and each of `held` released. Then it waits for the
/// refining slot, and once watch#13's plan frees it, a fresh refiner is started
/// on it.
fn moves_back_to_new(mut l: Logged, running: &[&str], held: &[&str]) {
    let effects = l.person(&move_to(twelve(), StageWord::New));
    let i = &l.w.issues[&twelve()];
    assert_eq!((i.stage, &i.group, &i.plan), (Stage::New, &None, &None));
    let stops: Vec<&Effect> = effects
        .iter()
        .filter(|e| matches!(e, Effect::StopAgent { .. }))
        .collect();
    assert_eq!(stops.len(), running.len(), "{effects:?}");
    for a in running {
        assert!(effects.contains(&stop(a)), "{a}: {effects:?}");
    }
    for f in held {
        assert!(effects.contains(&release(f)), "{f}: {effects:?}");
    }
    assert!(
        l.w.folders
            .values()
            .all(|f| f.holder.as_ref() != Some(&twelve()))
    );
    assert_eq!((&i.folder, i.other_folders.len()), (&None, 0));
    assert!(i.pulls.is_empty() && i.branches.is_empty() && i.closed.is_empty());
    assert!(i.fix.is_empty() && i.reviews.is_empty());
    assert_eq!(
        (i.round, i.review_ok, i.checks_ok, i.review_waived, i.told),
        (0, false, None, false, false)
    );
    assert!(!starts_on(&effects, &twelve()), "{effects:?}");
    let effects = l.apply(plan_of(watch("13")));
    assert!(effects.contains(&refiner_of(twelve())), "{effects:?}");
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Refining);
    let fresh =
        l.w.launch(&group(), Role::Refiner, &twelve(), None)
            .expect("a refiner's launch")
            .agent;
    assert!(!running.contains(&fresh.0.as_str()), "{fresh}");
    assert!(l.w.running(&twelve(), Role::Refiner).is_none());
    l.replayed();
}

#[test]
fn refining_moves_back_to_new_and_waits_for_a_slot() {
    let mut l = refining();
    // watch#13 is first in the tracker's order, so it takes the slot watch#12
    // leaves; watch#12 is taken again in its turn.
    let effects = l.person(&move_to(twelve(), StageWord::New));
    assert!(effects.contains(&refiner_of(watch("13"))), "{effects:?}");
    assert!(effects.contains(&stop("refiner-1")), "{effects:?}");
    assert!(!starts_on(&effects, &twelve()), "{effects:?}");
    assert_eq!(l.w.issues[&twelve()].stage, Stage::New);
    let effects = l.apply(plan_of(watch("13")));
    assert!(effects.contains(&refiner_of(twelve())), "{effects:?}");
    l.replayed();
}

#[test]
fn ready_moves_back_to_new() {
    moves_back_to_new(ready(), &[], &[]);
}

#[test]
fn working_moves_back_to_new() {
    moves_back_to_new(working(&["w1"]), &["author-1"], &["w1"]);
}

#[test]
fn in_review_moves_back_to_new() {
    moves_back_to_new(in_review(), &["author-1", "reviewer-1"], &["w1"]);
}

#[test]
fn fixing_moves_back_to_new() {
    moves_back_to_new(fixing(), &["author-1", "reviewer-1"], &["w1"]);
}

#[test]
fn with_the_arbiter_moves_back_to_new() {
    moves_back_to_new(
        arbitration(),
        &["author-1", "reviewer-1", "arbiter-1"],
        &["w1"],
    );
}

/// What a move of watch#12 back to ready must leave, with w1 its project's one
/// folder: at ready with its plan and group, each check waiting, round 0 and
/// nothing of its change; each of `running` stopped and w1 released. No author
/// is placed in w1 until author-1 has ended, and then a fresh one is.
fn moves_back_to_ready(mut l: Logged, running: &[&str]) {
    let effects = l.person(&move_to(twelve(), StageWord::Ready));
    let i = &l.w.issues[&twelve()];
    assert_eq!((i.stage, &i.group), (Stage::Ready, &Some(group())));
    let checks = &i.plan.as_ref().expect("the plan is kept").checks;
    assert_eq!(checks.len(), 1);
    assert!(checks.iter().all(|c| c.state == Some(CheckState::Waiting)));
    for a in running {
        assert!(effects.contains(&stop(a)), "{a}: {effects:?}");
    }
    assert!(effects.contains(&release("w1")), "{effects:?}");
    assert_eq!(l.w.folders[&FolderId::new("w1")].holder, None);
    assert_eq!(i.folder, None);
    assert!(i.pulls.is_empty() && i.branches.is_empty() && i.closed.is_empty());
    assert!(i.fix.is_empty() && i.reviews.is_empty());
    assert_eq!(
        (i.round, i.review_ok, i.checks_ok, i.review_waived),
        (0, false, None, false)
    );
    assert!(!starts_on(&effects, &twelve()), "{effects:?}");
    // The old author may still change w1 until it has ended.
    assert!(l.w.missing().start.contains(&stop("author-1")));
    let effects = l.apply(ended("author-1"));
    assert!(effects.contains(&author_of(twelve(), "w1")), "{effects:?}");
    assert!(!l.w.agents[&agent("author-1")].ended_waited_on);
    let fresh =
        l.w.launch(
            &group(),
            Role::Author,
            &twelve(),
            Some(&FolderId::new("w1")),
        )
        .expect("an author's launch")
        .agent;
    assert!(!running.contains(&fresh.0.as_str()), "{fresh}");
    l.replayed();
}

#[test]
fn working_moves_back_to_ready() {
    moves_back_to_ready(working(&["w1"]), &["author-1"]);
}

#[test]
fn in_review_moves_back_to_ready() {
    moves_back_to_ready(in_review(), &["author-1", "reviewer-1"]);
}

#[test]
fn fixing_moves_back_to_ready() {
    moves_back_to_ready(fixing(), &["author-1", "reviewer-1"]);
}

#[test]
fn with_the_arbiter_moves_back_to_ready() {
    moves_back_to_ready(arbitration(), &["author-1", "reviewer-1", "arbiter-1"]);
}

#[test]
fn back_to_ready_starts_a_new_author_only_once_the_old_one_has_ended() {
    let mut l = working(&["w1"]);
    let effects = l.person(&move_to(twelve(), StageWord::Ready));
    assert!(!starts_on(&effects, &twelve()), "{effects:?}");
    // Other events, and every round, place no author while author-1 runs.
    assert!(!starts_on(&l.apply(synced(watch("14"), 9)), &twelve()));
    assert!(!starts_on(&l.w.missing().start, &twelve()));
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Ready);
    assert_eq!(l.apply(ended("author-1")), [author_of(twelve(), "w1")]);
}

/// The area guild of ui and api, one folder in each, and the group watch.
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
            group: group(),
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

fn gld() -> IssueId {
    IssueId::new("guild", "GLD-12")
}

/// GLD-12 changing ui and api, with pull requests 3 in ui and 4 in api in
/// review.
fn guild_in_review() -> Logged {
    let mut l = guild();
    l.apply(synced(gld(), 1));
    l.apply(Event::PlanWritten {
        issue: gld(),
        plan: Plan {
            repos: vec![pid("ui"), pid("api")],
            ..plan()
        },
    });
    l.apply(started("author-1", Role::Author, gld(), Some("ui-1")));
    l.apply(Event::PullRequestReady {
        issue: gld(),
        pull: None,
        pulls: [(pid("ui"), "3".into()), (pid("api"), "4".into())].into(),
        replies: vec![],
    });
    assert_eq!(l.w.issues[&gld()].stage, Stage::InReview { round: 1 });
    l
}

/// GLD-12 back with its author after ui's pull request merged and api's was
/// refused.
fn guild_half_merged() -> Logged {
    let mut l = guild_in_review();
    l.apply(Event::ReviewFinished {
        issue: gld(),
        round: 1,
        verdict: Verdict::Approve,
        failed: Some(vec![]),
        open: Some(vec![]),
    });
    l.apply(Event::ChecksFinished {
        issue: gld(),
        passed: true,
        failing: vec![],
    });
    l.apply(Event::Merged {
        issue: gld(),
        repo: Some(pid("ui")),
    });
    l.apply(Event::MergeRefused {
        issue: gld(),
        reason: "it conflicts with main".into(),
    });
    let i = &l.w.issues[&gld()];
    assert!(matches!(i.stage, Stage::Fixing { .. }) && i.merged.contains(&pid("ui")));
    l
}

#[test]
fn every_other_move_is_refused_with_its_reason_and_becomes_no_event() {
    let cases: Vec<(Logged, IssueId, StageWord, &[&str])> = vec![
        (
            working(&["w1"]),
            twelve(),
            StageWord::Working,
            &["watch#12 is already working"],
        ),
        (
            refining(),
            watch("13"),
            StageWord::Ready,
            &["watch#13 is new and has no plan yet"],
        ),
        (
            refining(),
            twelve(),
            StageWord::Ready,
            &["watch#12 is refining and has no plan yet"],
        ),
        (
            working(&["w1"]),
            twelve(),
            StageWord::Refining,
            &[
                "the loop takes an issue at new for refining",
                "move it to new",
            ],
        ),
        (
            ready(),
            twelve(),
            StageWord::Working,
            &[
                "the loop starts an author on a ready issue",
                "move it to ready",
            ],
        ),
        (
            working(&["w1"]),
            twelve(),
            StageWord::InReview,
            &["its author's ready puts it in review"],
        ),
        (
            in_review(),
            twelve(),
            StageWord::Fixing,
            &["sends it back to its author"],
        ),
        (
            in_review(),
            twelve(),
            StageWord::Arbitration,
            &["the loop hands it to an arbiter"],
        ),
        (
            in_review(),
            twelve(),
            StageWord::Merging,
            &["the loop merges it once its review and its checks have passed"],
        ),
        (
            fixing(),
            twelve(),
            StageWord::Deploying,
            &["the loop deploys and verifies it after its merge"],
        ),
        (
            merging(),
            twelve(),
            StageWord::New,
            &["watch#12 is merging", "the loop cannot undo a merge"],
        ),
        (
            deploying(),
            twelve(),
            StageWord::New,
            &["watch#12 is deploying", "the loop cannot undo a merge"],
        ),
        (
            verifying(),
            twelve(),
            StageWord::Ready,
            &["watch#12 is verifying", "the loop cannot undo a merge"],
        ),
        (
            guild_half_merged(),
            gld(),
            StageWord::New,
            &[
                "has its pull request in ui merged",
                "the loop cannot undo a merge",
            ],
        ),
        (
            working(&["w1"]),
            twelve(),
            StageWord::Done,
            &["close it in its tracker"],
        ),
        (
            done(),
            twelve(),
            StageWord::New,
            &["watch#12 is done", "reopen it in its tracker"],
        ),
        (
            working(&["w1"]),
            twelve(),
            StageWord::Unknown,
            &[
                "a stage this build knows: new, refining, ready, working, in_review, fixing, arbitration, merging, deploying, verifying, done",
            ],
        ),
    ];
    for (l, issue, to, words) in cases {
        let before = l.w.clone();
        let why = l.refused(None, &move_to(issue.clone(), to));
        for w in words {
            assert!(why.contains(w), "{issue} to {}: {why}", to.name());
        }
        assert_eq!(l.w, before);
    }
}

#[test]
fn an_unknown_issue_is_refused() {
    let l = working(&["w1"]);
    assert_eq!(
        l.refused(None, &move_to(watch("99"), StageWord::New)),
        "there is no issue watch#99"
    );
}

#[test]
fn an_agent_may_not_move_an_issue() {
    let l = working(&["w1"]);
    let why = l.refused(Some("author-1"), &move_to(twelve(), StageWord::Ready));
    assert!(
        why.starts_with("move is a person's: author-1 is an agent"),
        "{why}"
    );
}

#[test]
fn the_view_lists_exactly_the_moves_the_command_takes() {
    let worlds = [
        refining(),
        ready(),
        working(&["w1"]),
        in_review(),
        fixing(),
        arbitration(),
        merging(),
        deploying(),
        verifying(),
        done(),
        guild_half_merged(),
    ];
    let words: Vec<StageWord> = StageWord::ALL
        .into_iter()
        .chain([StageWord::Unknown])
        .collect();
    for l in &worlds {
        let view = View::new(&l.w, &Default::default());
        for i in l.w.issues.values() {
            let listed = l.w.moves_of(i);
            for to in &words {
                let taken = l.w.check(None, &move_to(i.id.clone(), *to)).is_ok();
                assert_eq!(listed.contains(to), taken, "{} to {}", i.id, to.name());
            }
            if let Some(shown) = view.issues.iter().find(|v| v.id == i.id.to_string()) {
                assert_eq!(shown.moves, listed, "{}", i.id);
            }
        }
    }
    let moves = |l: &Logged, id: &IssueId| l.w.moves_of(&l.w.issues[id]);
    use StageWord::{New, Ready};
    assert_eq!(
        moves(&refining(), &watch("13")),
        [],
        "new, with nothing to undo"
    );
    assert_eq!(moves(&refining(), &twelve()), [New]);
    assert_eq!(moves(&ready(), &twelve()), [New]);
    assert_eq!(moves(&working(&["w1"]), &twelve()), [New, Ready]);
    assert_eq!(moves(&in_review(), &twelve()), [New, Ready]);
    assert_eq!(moves(&fixing(), &twelve()), [New, Ready]);
    assert_eq!(moves(&arbitration(), &twelve()), [New, Ready]);
    for l in [merging(), deploying(), verifying(), done()] {
        assert_eq!(moves(&l, &twelve()), []);
    }
    assert_eq!(moves(&guild_half_merged(), &gld()), []);
}

#[test]
fn the_reply_says_the_new_stage_and_names_each_pull_request_left_open() {
    let reply = |mut l: Logged, issue: IssueId, to: StageWord| {
        let dropped = l.w.open_pulls(&issue);
        l.person(&move_to(issue.clone(), to));
        crate::text::moved(&l.w, &issue, &dropped)
    };
    assert_eq!(
        reply(working(&["w1"]), twelve(), StageWord::Ready),
        "watch#12 is ready"
    );
    assert_eq!(
        reply(in_review(), twelve(), StageWord::New),
        "watch#12 is new; the move dropped its pull request 57, which is left open on the code \
         host for you to close or reuse"
    );
    assert_eq!(
        reply(guild_in_review(), gld(), StageWord::Ready),
        "guild#GLD-12 is ready; the move dropped its pull requests 3 in ui and 4 in api, which \
         are left open on the code host for you to close or reuse"
    );
    // Taken for refining at once, it says so.
    let mut l = watch_project(&["w1"]);
    l.apply(synced(twelve(), 5));
    assert_eq!(reply(l, twelve(), StageWord::New), "watch#12 is refining");
}

#[test]
fn decisions_settings_and_what_a_failed_verify_left_stay() {
    let mut l = working(&["w1"]);
    l.run(
        "author-1",
        &Command::Choose {
            issue: twelve(),
            question: "Which field carries the tower?".into(),
            options: vec!["a header".into(), "the body".into()],
            took: "a header".into(),
            why: "the clacks read headers".into(),
            cost: Some("low".into()),
        },
    );
    l.apply(Event::SettingSet {
        scope: Scope::Task(twelve()),
        role: None,
        key: "starts".into(),
        value: Value::Number(5),
        by: "person".into(),
    });
    // Merged, deployed and failed its verify: back at ready with no author,
    // since author-1's stop after the merge has not worked.
    l.apply(pull_ready(&[("watch", "57")], &[]));
    l.apply(verdict(1, Verdict::Approve, vec![]));
    l.apply(Event::ChecksFinished {
        issue: twelve(),
        passed: true,
        failing: vec![],
    });
    l.apply(Event::Merged {
        issue: twelve(),
        repo: Some(pid("watch")),
    });
    l.apply(Event::Deployed { issue: twelve() });
    l.apply(Event::Verified {
        issue: twelve(),
        passed: false,
        output: "the tower is missing".into(),
    });
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Ready);
    let decisions = l.w.decisions.clone();
    l.person(&move_to(twelve(), StageWord::New));
    let i = &l.w.issues[&twelve()];
    assert_eq!(i.stage, Stage::New);
    assert!(i.verify_failed && i.reopen_due && i.has_merged);
    assert!(i.fix.is_empty(), "its fix points go: {:?}", i.fix);
    assert_eq!(l.w.decisions, decisions);
    assert_eq!(l.w.setting_number(&twelve(), None, "starts"), 5);
    l.replayed();
}

#[test]
fn an_old_refiner_whose_stop_fails_is_asked_again_while_a_fresh_one_refines() {
    let mut l = watch_project(&["w1"]);
    l.apply(synced(twelve(), 5));
    l.apply(started("refiner-1", Role::Refiner, twelve(), None));
    // The only issue at new: taken for refining again at once.
    let effects = l.person(&move_to(twelve(), StageWord::New));
    assert!(effects.contains(&stop("refiner-1")), "{effects:?}");
    assert!(effects.contains(&refiner_of(twelve())), "{effects:?}");
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Refining);
    // refiner-1's stop failed and the fresh refiner's start was lost: both are
    // asked for again.
    let missing = l.w.missing();
    assert!(missing.start.contains(&stop("refiner-1")), "{missing:?}");
    assert!(missing.start.contains(&refiner_of(twelve())), "{missing:?}");
    l.apply(started("refiner-2", Role::Refiner, twelve(), None));
    assert_eq!(l.w.missing().start, [stop("refiner-1")]);
    assert_eq!(l.replayed().missing().start, [stop("refiner-1")]);
    assert_eq!(
        l.w.running(&twelve(), Role::Refiner).map(|a| a.id.clone()),
        Some(agent("refiner-2"))
    );
    let old = &l.w.agents[&agent("refiner-1")];
    assert!(!l.w.waiting_on(old), "it is not reminded");
    assert!(l.w.waiting_on(&l.w.agents[&agent("refiner-2")]));
    let planned = Command::Plan {
        issue: twelve(),
        plan: plan(),
    };
    let why = l.refused(Some("refiner-1"), &planned);
    assert!(
        why.contains("a person moved watch#12 to new after refiner-1 started")
            && why.contains("refiner-2 is working on it now"),
        "{why}"
    );
    let chose = Command::Choose {
        issue: twelve(),
        question: "Which tower?".into(),
        options: vec!["Hub".into(), "Genua".into()],
        took: "Hub".into(),
        why: "it is nearer".into(),
        cost: Some("low".into()),
    };
    let why = l.refused(Some("refiner-1"), &chose);
    assert!(why.contains("a person moved watch#12 to new"), "{why}");
    let set = Command::Set {
        target: crate::settings::Target::Issue(twelve()),
        role: None,
        key: "model".into(),
        value: "opus".into(),
    };
    let why = l.refused(Some("refiner-1"), &set);
    assert!(why.contains("a person moved watch#12 to new"), "{why}");
    l.run("refiner-2", &planned);
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Working);
}

#[test]
fn an_old_author_is_refused_and_stopped_again_while_a_new_one_works_in_another_folder() {
    let mut l = working(&["w1", "w2"]);
    let effects = l.person(&move_to(twelve(), StageWord::Ready));
    assert!(effects.contains(&stop("author-1")), "{effects:?}");
    assert!(effects.contains(&release("w1")), "{effects:?}");
    // w1 stays with author-1 until it has ended; the new author takes w2.
    assert!(effects.contains(&author_of(twelve(), "w2")), "{effects:?}");
    l.apply(started("author-2", Role::Author, twelve(), Some("w2")));
    assert_eq!(l.w.folders[&FolderId::new("w1")].holder, None);
    let ready = Command::Ready {
        issue: twelve(),
        pull: Some("58".into()),
        pulls: Default::default(),
        replies: vec![],
    };
    let why = l.refused(Some("author-1"), &ready);
    assert!(
        why.contains("a person moved watch#12 to ready after author-1 started")
            && why.contains("author-2 is working on it now"),
        "{why}"
    );
    for w in [&l.w, &l.replayed()] {
        let start = w.missing().start;
        assert!(start.contains(&stop("author-1")), "{start:?}");
        assert!(!starts_on(&start, &twelve()), "{start:?}");
        assert_eq!(
            w.running(&twelve(), Role::Author).map(|a| a.id.clone()),
            Some(agent("author-2"))
        );
    }
    l.run("author-2", &ready);
    assert_eq!(l.w.issues[&twelve()].stage, Stage::InReview { round: 1 });
}

#[test]
fn an_old_reviewer_hears_no_round_and_an_old_arbiter_no_dispute() {
    let mut l = arbitration();
    l.person(&move_to(twelve(), StageWord::Ready));
    // The new author's change is in review: the next round is for a fresh
    // reviewer, and its block for a fresh arbiter.
    l.apply(ended("author-1"));
    l.apply(started("author-2", Role::Author, twelve(), Some("w1")));
    let effects = l.apply(pull_ready(&[("watch", "61")], &[]));
    assert!(effects.contains(&Effect::StartReview {
        issue: twelve(),
        round: 1
    }));
    assert!(l.w.running(&twelve(), Role::Reviewer).is_none());
    let review = Command::Review {
        issue: twelve(),
        verdict: Verdict::Approve,
        failed: vec![],
        open: vec![],
    };
    let why = l.refused(Some("reviewer-1"), &review);
    assert!(why.contains("a person moved watch#12 to ready"), "{why}");
    l.apply(started("reviewer-2", Role::Reviewer, twelve(), None));
    l.apply(verdict(1, block("no tower"), vec![1]));
    l.apply(pull_ready(&[], &["there is now"]));
    l.apply(Event::Told { issue: twelve() });
    let effects = l.apply(verdict(2, block("still none"), vec![1]));
    let arbiter = Effect::StartAgent {
        group: group(),
        role: Role::Arbiter,
        issue: twelve(),
        folder: None,
    };
    assert!(effects.contains(&arbiter), "{effects:?}");
    let rule = Command::Rule {
        issue: twelve(),
        approve: true,
        reason: "good enough".into(),
    };
    let why = l.refused(Some("arbiter-1"), &rule);
    assert!(why.contains("a person moved watch#12 to ready"), "{why}");
    let start = l.w.missing().start;
    for old in ["reviewer-1", "arbiter-1"] {
        assert!(start.contains(&stop(old)), "{old}: {start:?}");
    }
}

#[test]
fn an_old_agent_that_ends_after_the_move_uses_no_start() {
    let mut l = watch_project(&["w1"]);
    l.apply(Event::SettingSet {
        scope: Scope::Project(pid("watch")),
        role: None,
        key: "starts".into(),
        value: Value::Number(1),
        by: "person".into(),
    });
    l.apply(synced(twelve(), 5));
    l.apply(started("refiner-1", Role::Refiner, twelve(), None));
    l.person(&move_to(twelve(), StageWord::New));
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Refining);
    l.apply(ended("refiner-1"));
    assert!(!l.w.agents[&agent("refiner-1")].ended_waited_on);
    let missing = l.w.missing();
    assert_eq!(missing.start, [refiner_of(twelve())]);
    assert!(missing.given_up.is_empty(), "{missing:?}");
}

#[test]
fn a_move_is_taken_while_held_and_starts_nothing_until_resumed() {
    let mut l = working(&["w1", "w2"]);
    l.apply(Event::Held);
    let effects = l.person(&move_to(twelve(), StageWord::Ready));
    assert!(effects.contains(&stop("author-1")), "{effects:?}");
    assert!(effects.contains(&release("w1")), "{effects:?}");
    assert!(!starts_on(&effects, &twelve()), "{effects:?}");
    assert_eq!(l.w.issues[&twelve()].stage, Stage::Ready);
    let start = l.w.missing().start;
    assert!(start.contains(&stop("author-1")), "{start:?}");
    assert!(!starts_on(&start, &twelve()), "{start:?}");
    let effects = l.apply(Event::Resumed);
    assert!(effects.contains(&author_of(twelve(), "w2")), "{effects:?}");
}

#[test]
fn the_move_is_in_the_log_and_a_replay_gives_the_same_world() {
    let mut l = in_review();
    l.person(&move_to(twelve(), StageWord::New));
    assert!(l.log.contains(&Event::Moved {
        issue: twelve(),
        stage: Stage::New,
    }));
    let w = l.replayed();
    assert_eq!(w.issues[&twelve()].stage, Stage::New);
    assert!(w.issues[&twelve()].moved.is_some());
    // With times, as the daemon replays its log.
    let timed: Vec<(u64, &Event)> = (1_000..).zip(&l.log).collect();
    let (at, _) = World::default().replay_at(timed.iter().copied());
    let mut live = World::default();
    for (ms, e) in &timed {
        live.apply_at(*ms, e);
    }
    assert_eq!(at, live);
}
