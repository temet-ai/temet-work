//! A project's `code_host` picks its adapter, whatever its tracker is, and a
//! GitLab project's repository is the full group path.

#![cfg(all(feature = "codehost-github", feature = "codehost-gitlab"))]

use std::sync::{Arc, Mutex};

use temet_work_codehost::CodeHosts;
use temet_work_core::ids::{IssueId, ProjectId};
use temet_work_core::model::Stage;
use temet_work_core::{Effect, Event, Outcome, Part, World};

type Seen = Arc<Mutex<Vec<String>>>;

struct Gh(Seen);

impl temet_work_codehost_github::Transport for Gh {
    fn gh(&mut self, args: &[&str]) -> Result<String, String> {
        self.0.lock().expect("lock").push(args.join(" "));
        Ok(r#"{"number":5,"state":"OPEN","mergeable":"MERGEABLE","statusCheckRollup":[{"__typename":"CheckRun","name":"rust","status":"COMPLETED","conclusion":"SUCCESS"}]}"#.into())
    }
}

struct Gl(Seen);

impl temet_work_codehost_gitlab::Transport for Gl {
    fn glab(&mut self, args: &[&str]) -> Result<String, String> {
        self.0.lock().expect("lock").push(args.join(" "));
        if args.iter().any(|a| a.contains("/jobs")) {
            return Ok(r#"[{"name":"unit-tests","status":"failed","allow_failure":false}]"#.into());
        }
        Ok(r#"{"iid":6,"state":"opened","has_conflicts":false,"pipeline":{"id":1,"status":"failed"},"head_pipeline":{"id":1,"status":"failed"}}"#.into())
    }
}

fn world() -> World {
    let mut w = World::default();
    // A Jira project with its code on GitHub, and a GitHub project with its code on GitLab.
    for (id, tracker, host, repo) in [
        ("watch", "jira", "github", "ankh-morpork/watch"),
        ("guild", "github", "gitlab", "ankh-morpork/guilds/thieves"),
    ] {
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new(id),
            tracker: tracker.into(),
            name: "NAME".into(),
            code_host: Some(host.into()),
            repo: Some(repo.into()),
        });
        let issue = IssueId::new(id, "1");
        w.apply(&Event::IssueSynced {
            issue: issue.clone(),
            title: "an issue".into(),
            milestone: None,
            depends_on: vec![],
            order: 1,
            closed: false,
        });
        if let Some(i) = w.issues.get_mut(&issue) {
            i.stage = Stage::InReview { round: 1 };
            let number = if id == "watch" { "5" } else { "6" };
            i.pulls.insert(ProjectId::new(id), number.into());
        }
    }
    w
}

#[test]
fn each_project_reads_its_pull_request_through_the_adapter_its_code_host_names() {
    let (gh, gl) = (Seen::default(), Seen::default());
    let mut part = CodeHosts::default();
    part.add(Box::new(temet_work_codehost_github::GitHub::with(
        Box::new(Gh(gh.clone())),
    )));
    part.add(Box::new(temet_work_codehost_gitlab::GitLab::with(
        Box::new(Gl(gl.clone())),
    )));
    assert_eq!(part.ids(), ["github", "gitlab"]);
    let polled = part.poll(&world(), 0);
    assert_eq!(polled.problem, None);
    assert_eq!(
        polled.events,
        vec![
            Event::ChecksFinished {
                issue: IssueId::new("guild", "1"),
                passed: false,
                failing: vec!["unit-tests".into()],
            },
            Event::ChecksFinished {
                issue: IssueId::new("watch", "1"),
                passed: true,
                failing: vec![],
            },
        ]
    );
    let gh = gh.lock().expect("lock").clone();
    let gl = gl.lock().expect("lock").clone();
    assert_eq!(gh.len(), 1);
    assert!(
        gh[0].starts_with("pr view 5 -R ankh-morpork/watch"),
        "{gh:?}"
    );
    assert_eq!(
        gl,
        [
            "api projects/ankh-morpork%2Fguilds%2Fthieves/merge_requests/6",
            "api projects/ankh-morpork%2Fguilds%2Fthieves/pipelines/1/jobs?scope[]=failed&per_page=100"
        ]
    );
}

/// A merged request whose main pipeline waits at a manual job.
struct Held;

impl temet_work_codehost_gitlab::Transport for Held {
    fn glab(&mut self, args: &[&str]) -> Result<String, String> {
        Ok(if args[1].contains("/jobs") {
            "[]".into()
        } else if args[1].contains("/pipelines?") {
            r#"[{"id":9,"status":"manual"}]"#.into()
        } else {
            r#"{"iid":1,"state":"merged","merge_commit_sha":"abc","target_branch":"main"}"#.into()
        })
    }
}

#[test]
fn a_main_pipeline_held_at_a_manual_job_is_listed_while_the_next_repository_waits() {
    let mut w = World::default();
    let (ui, api, area) = (
        ProjectId::new("ui"),
        ProjectId::new("api"),
        ProjectId::new("guild"),
    );
    w.apply(&Event::AreaDefined {
        area: area.clone(),
        name: "Guild".into(),
        projects: vec![ui.clone(), api.clone()],
        tracker: "jira".into(),
        tracker_name: "GLD".into(),
    });
    for id in [&ui, &api] {
        w.apply(&Event::ProjectAdded {
            project: id.clone(),
            tracker: String::new(),
            name: format!("guild/{id}"),
            code_host: Some("gitlab".into()),
            repo: None,
        });
    }
    let issue = IssueId::new("guild", "GLD-1");
    w.apply(&Event::IssueSynced {
        issue: issue.clone(),
        title: "an issue".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    });
    if let Some(i) = w.issues.get_mut(&issue) {
        i.stage = Stage::Merging;
        i.plan = Some(temet_work_core::model::Plan {
            repos: vec![ui.clone(), api.clone()],
            ..Default::default()
        });
        i.pulls = [(ui.clone(), "1".to_string()), (api, "2".to_string())].into();
        i.merged.insert(ui);
    }
    let mut part = CodeHosts::default();
    part.add(Box::new(temet_work_codehost_gitlab::GitLab::with(
        Box::new(Held),
    )));
    let outcome = part.carry_out(&Effect::Merge { issue }, &w);
    assert!(
        matches!(&outcome, Outcome::Failed(why) if why.contains("pipeline 9 on main is held at a manual job")),
        "{outcome:?}"
    );
}
