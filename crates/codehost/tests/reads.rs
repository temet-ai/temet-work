//! `change`, `show`, `search` and `ci` through the code-host family, against a
//! bare repository each test makes on disk as the code host's origin. The family
//! keeps one shared copy of it, which these tests look into and count the git
//! commands of.

use std::path::{Path, PathBuf};
use std::process::Command as Git;

use temet_work_codehost::{CheckRun, Checks, CodeHosts, FakeCodeHost, Pull, PullState, RunState};
use temet_work_core::ids::{IssueId, ProjectId};
use temet_work_core::model::{Plan, Stage};
use temet_work_core::{Answer, At, Command, Event, Part, World};

/// A folder of its own for test `name`, empty.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("reads-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch folder");
    dir
}

/// Runs git in `dir` with a fixed author and none of the person's hooks or
/// signing, and gives what it printed.
fn git(dir: &Path, args: &[&str]) -> String {
    let out = Git::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.name=Sam Vimes",
            "-c",
            "user.email=vimes@example.com",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A code host's repository: a bare origin the test writes commits into
/// straight, as pushes would land, with `git fast-import`: one git command per
/// commit.
struct Origin {
    bare: PathBuf,
    made: std::cell::Cell<u32>,
}

impl Origin {
    fn new(dir: &Path, name: &str) -> Self {
        let bare = dir.join(format!("{name}.git"));
        std::fs::create_dir_all(&bare).expect("bare");
        git(&bare, &["init", "--bare", "--quiet", "-b", "main"]);
        Self {
            bare,
            made: std::cell::Cell::new(0),
        }
    }

    /// Commits `files` with `message` on top of `from` (none for a first
    /// commit), points the ref `to` at it, and gives its id.
    fn commit(
        &self,
        from: Option<&str>,
        files: &[(&str, &str)],
        message: &str,
        to: &str,
    ) -> String {
        self.import(&[(from, files, message, to)]).remove(0)
    }

    /// Makes each commit as [`Origin::commit`] does, all in one `fast-import`,
    /// and gives their ids in order. A `from` of `:N` is the Nth commit of this
    /// list, counted from 1.
    fn import(&self, commits: &[(Option<&str>, &[(&str, &str)], &str, &str)]) -> Vec<String> {
        let mut stream = String::new();
        for (i, (from, files, message, to)) in commits.iter().enumerate() {
            let n = self.made.get() + 1;
            self.made.set(n);
            let when = 1_700_000_000 + u64::from(n) * 60;
            stream.push_str(&format!(
                "commit {to}\nmark :{}\nauthor Sam Vimes <vimes@example.com> {when} +0000\n\
                 committer Sam Vimes <vimes@example.com> {when} +0000\ndata {}\n{message}\n",
                i + 1,
                message.len()
            ));
            if let Some(parent) = from {
                stream.push_str(&format!("from {parent}\n"));
            }
            for (path, text) in *files {
                stream.push_str(&format!(
                    "M 100644 inline {path}\ndata {}\n{text}\n",
                    text.len()
                ));
            }
            stream.push('\n');
        }
        let marks = self.bare.join(format!("marks-{}", self.made.get()));
        let mut child = Git::new("git")
            .arg("-C")
            .arg(&self.bare)
            .args(["fast-import", "--quiet", "--force"])
            .arg(format!("--export-marks={}", marks.display()))
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("fast-import");
        {
            use std::io::Write;
            child
                .stdin
                .take()
                .expect("stdin")
                .write_all(stream.as_bytes())
                .expect("stream");
        }
        assert!(child.wait().expect("waits").success(), "fast-import");
        let marked = std::fs::read_to_string(&marks).expect("marks");
        let _ = std::fs::remove_file(&marks);
        let mut ids: Vec<(usize, String)> = marked
            .lines()
            .filter_map(|l| {
                let (mark, id) = l.split_once(' ')?;
                Some((mark.trim_start_matches(':').parse().ok()?, id.to_string()))
            })
            .collect();
        ids.sort();
        ids.into_iter().map(|(_, id)| id).collect()
    }
}

fn watch_12() -> IssueId {
    IssueId::new("watch", "12")
}

/// A world with project `id` on code host "fake" for each of `repos`, as (id,
/// repository name), and watch#12 in review with pull request 57 in each.
fn world(repos: &[(&str, &str)]) -> World {
    let mut w = World::default();
    for (id, name) in repos {
        w.apply(&Event::ProjectAdded {
            project: ProjectId::new(*id),
            tracker: "fake".into(),
            name: (*name).into(),
            code_host: None,
            repo: None,
        });
    }
    w.apply(&Event::IssueSynced {
        issue: watch_12(),
        title: "A night shift rota".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    });
    let i = w.issues.get_mut(&watch_12()).expect("issue");
    i.stage = Stage::InReview { round: 1 };
    for (id, _) in repos {
        i.pulls.insert(ProjectId::new(*id), "57".into());
    }
    w
}

fn city_12() -> IssueId {
    IssueId::new("city", "12")
}

/// A world with area city of projects watch and guild, and city#12 in review
/// with pull request 57 in each, its plan naming watch first.
fn city() -> World {
    let mut w = world(&[
        ("watch", "ankh-morpork/watch"),
        ("guild", "ankh-morpork/guild"),
    ]);
    w.apply(&Event::AreaDefined {
        area: ProjectId::new("city"),
        name: "The city".into(),
        projects: vec![ProjectId::new("watch"), ProjectId::new("guild")],
        tracker: "fake".into(),
        tracker_name: "CITY".into(),
    });
    w.apply(&Event::IssueSynced {
        issue: city_12(),
        title: "A night shift for the guild too".into(),
        milestone: None,
        depends_on: vec![],
        order: 2,
        closed: false,
    });
    let i = w.issues.get_mut(&city_12()).expect("issue");
    i.stage = Stage::InReview { round: 1 };
    i.plan = Some(Plan {
        repos: vec![ProjectId::new("watch"), ProjectId::new("guild")],
        ..Plan::default()
    });
    for id in ["guild", "watch"] {
        i.pulls.insert(ProjectId::new(id), "57".into());
    }
    w
}

/// The family with a fake code host whose repositories clone from `origins`,
/// as (name, origin), keeping its copies under `dir`.
fn family(dir: &Path, origins: &[(&str, &Origin)]) -> CodeHosts {
    let mut host = FakeCodeHost::default();
    for (name, origin) in origins {
        host.clones
            .insert((*name).into(), origin.bare.to_string_lossy().into_owned());
        host.pulls.push((
            (*name).into(),
            Pull {
                number: "57".into(),
                state: PullState::Open,
                checks: Checks::Pending,
                conflicting: false,
                draft: false,
                branch: "watch-12-night-shift".into(),
            },
        ));
    }
    let mut part = CodeHosts::default().copies(dir.join("copies"));
    part.add(Box::new(host));
    part
}

/// The text `command` answers with, or its refusal.
fn read(part: &mut CodeHosts, w: &World, command: Command) -> Result<String, String> {
    part.read(w, &command)
        .expect("the code-host family answers it")
        .and_then(Answer::wait)
}

fn show(project: &str, path: &str, at: Option<At>) -> Command {
    Command::Show {
        project: ProjectId::new(project),
        path: path.into(),
        at,
    }
}

fn search(pattern: &str, at: Option<At>) -> Command {
    Command::Search {
        project: ProjectId::new("watch"),
        pattern: pattern.into(),
        at,
    }
}

/// How many git commands so far began with `word`.
fn count(part: &CodeHosts, word: &str) -> usize {
    let copies = part.copies_kept().expect("copies");
    let ran = copies.ran.lock().expect("ran");
    ran.iter()
        .filter(|a| a.first().map(String::as_str) == Some(word))
        .count()
}

/// The watch repository: main with a README and a rota, then branch
/// watch-12-night-shift pushed as pull request 57 with two commits, and main
/// moved on by one more commit after the branch left it.
struct Watch {
    origin: Origin,
    first: String,
    night: String,
    moved: String,
}

fn watch(dir: &Path) -> Watch {
    let origin = Origin::new(dir, "watch");
    let ids = origin.import(&[
        (
            None,
            &[
                ("README.md", "The Night Watch\n"),
                ("src/rota.rs", "pub fn rota() -> &'static str { \"day\" }\n"),
                ("src/bell.rs", "pub fn ring() {}\n"),
            ],
            "The watch house",
            "refs/heads/main",
        ),
        (
            Some(":1"),
            &[(
                "src/rota.rs",
                "pub fn rota() -> &'static str { \"night\" }\n",
            )],
            "A night shift on the rota",
            "refs/pull/57/head",
        ),
        (
            Some(":2"),
            &[("src/lamp.rs", "pub fn light() {}\n")],
            "A lamp for the night",
            "refs/pull/57/head",
        ),
        (
            Some(":1"),
            &[("README.md", "The City Watch\n")],
            "Rename the watch",
            "refs/heads/main",
        ),
    ]);
    Watch {
        origin,
        first: ids[0].clone(),
        night: ids[2].clone(),
        moved: ids[3].clone(),
    }
}

#[test]
fn readers_share_one_copy_a_second_read_clones_nothing_and_it_has_no_working_tree() {
    let dir = scratch("one-copy");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    for _ in 0..2 {
        assert_eq!(
            read(&mut part, &w, show("watch", "README.md", None)).as_deref(),
            Ok("The City Watch\n")
        );
    }
    assert_eq!(count(&part, "clone"), 1, "the second read clones nothing");
    let copy = dir.join("copies/fake/ankh-morpork/watch.git");
    assert_eq!(git(&copy, &["rev-parse", "--is-bare-repository"]), "true");
    let names: Vec<String> = std::fs::read_dir(&copy)
        .expect("copy")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    for checked_out in ["README.md", "src", ".git", "target"] {
        assert!(
            !names.iter().any(|n| n == checked_out),
            "{checked_out} in {names:?}"
        );
    }
    // Only the copies folder and the test's own repositories are there.
    assert!(!dir.join("copies/fake/ankh-morpork/watch.git.part").exists());
}

#[test]
fn show_prints_a_file_at_the_default_head_a_pull_requests_head_and_a_commit() {
    let dir = scratch("show-points");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    let rota = |part: &mut CodeHosts, at| read(part, &w, show("watch", "src/rota.rs", at));
    assert_eq!(
        rota(&mut part, None).as_deref(),
        Ok("pub fn rota() -> &'static str { \"day\" }\n")
    );
    assert_eq!(
        rota(&mut part, Some(At::Issue(watch_12()))).as_deref(),
        Ok("pub fn rota() -> &'static str { \"night\" }\n")
    );
    assert_eq!(
        read(
            &mut part,
            &w,
            show(
                "watch",
                "README.md",
                Some(At::Commit(repo.first[..10].into()))
            )
        )
        .as_deref(),
        Ok("The Night Watch\n")
    );
}

#[test]
fn show_lists_a_directory_with_a_slash_after_each_folder() {
    let dir = scratch("show-dir");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    assert_eq!(
        read(&mut part, &w, show("watch", "", None)).as_deref(),
        Ok("README.md\nsrc/\n")
    );
    assert_eq!(
        read(
            &mut part,
            &w,
            show("watch", "src/", Some(At::Issue(watch_12())))
        )
        .as_deref(),
        Ok("bell.rs\nlamp.rs\nrota.rs\n")
    );
}

#[test]
fn show_refuses_a_path_a_commit_or_a_pull_request_the_repository_does_not_have() {
    let dir = scratch("show-refused");
    let repo = watch(&dir);
    let mut w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    let refused =
        |part: &mut CodeHosts, w: &World, command| read(part, w, command).expect_err("refused");
    let why = refused(&mut part, &w, show("watch", "src/lamp.rs", None));
    assert_eq!(
        why,
        "watch (ankh-morpork/watch) has no src/lamp.rs at the default branch's head"
    );
    let why = refused(
        &mut part,
        &w,
        show("watch", "README.md", Some(At::Commit("0123abcd".into()))),
    );
    assert_eq!(why, "watch (ankh-morpork/watch) has no commit 0123abcd");
    let why = refused(
        &mut part,
        &w,
        show("watch", "README.md", Some(At::Commit("main".into()))),
    );
    assert!(why.starts_with("main is not a commit id"), "{why}");
    // A project the world has, with no repository on any code host here.
    w.apply(&Event::ProjectAdded {
        project: ProjectId::new("lancre"),
        tracker: "fake".into(),
        name: "hills/lancre".into(),
        code_host: Some("nonesuch".into()),
        repo: None,
    });
    let why = refused(&mut part, &w, show("lancre", "README.md", None));
    assert_eq!(why, "no code host \"nonesuch\" in this build");
    // An issue with no pull request in the project.
    w.issues.get_mut(&watch_12()).expect("issue").pulls.clear();
    let why = refused(
        &mut part,
        &w,
        show("watch", "README.md", Some(At::Issue(watch_12()))),
    );
    assert_eq!(why, "watch#12 has no pull request in watch on record");
}

#[test]
fn a_read_at_a_head_shows_a_commit_pushed_just_before_it() {
    let dir = scratch("fresh-heads");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    let lamp = |part: &mut CodeHosts| {
        read(
            part,
            &w,
            show("watch", "src/lamp.rs", Some(At::Issue(watch_12()))),
        )
    };
    assert_eq!(lamp(&mut part).as_deref(), Ok("pub fn light() {}\n"));
    assert_eq!(
        read(&mut part, &w, show("watch", "README.md", None)).as_deref(),
        Ok("The City Watch\n")
    );
    // The author pushes to the pull request, and someone merges to main.
    repo.origin.commit(
        Some(&repo.night),
        &[("src/lamp.rs", "pub fn light() { trim() }\n")],
        "Trim the lamp",
        "refs/pull/57/head",
    );
    repo.origin.commit(
        Some(&repo.moved),
        &[("README.md", "The City Watch, Pseudopolis Yard\n")],
        "Move house",
        "refs/heads/main",
    );
    assert_eq!(
        lamp(&mut part).as_deref(),
        Ok("pub fn light() { trim() }\n")
    );
    assert_eq!(
        read(&mut part, &w, show("watch", "README.md", None)).as_deref(),
        Ok("The City Watch, Pseudopolis Yard\n")
    );
}

#[test]
fn a_read_at_a_commit_the_copy_holds_fetches_nothing() {
    let dir = scratch("held-commit");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    read(&mut part, &w, show("watch", "README.md", None)).expect("first read");
    let fetches = count(&part, "fetch");
    for commit in [&repo.first, &repo.moved] {
        read(
            &mut part,
            &w,
            show("watch", "README.md", Some(At::Commit(commit.clone()))),
        )
        .expect("held");
        read(
            &mut part,
            &w,
            search("watch", Some(At::Commit(commit.clone()))),
        )
        .expect("held");
    }
    assert_eq!(
        count(&part, "fetch"),
        fetches,
        "nothing fetched for a held commit"
    );
    // One the copy does not hold yet is fetched.
    let later = repo.origin.commit(
        Some(&repo.moved),
        &[("src/bell.rs", "pub fn ring() { clang() }\n")],
        "Clang",
        "refs/heads/main",
    );
    assert_eq!(
        read(
            &mut part,
            &w,
            show("watch", "src/bell.rs", Some(At::Commit(later)))
        )
        .as_deref(),
        Ok("pub fn ring() { clang() }\n")
    );
    assert!(count(&part, "fetch") > fetches);
}

#[test]
fn search_prints_each_matching_line_at_each_point_and_says_so_when_none_match() {
    let dir = scratch("search");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    assert_eq!(
        read(&mut part, &w, search("pub fn (rota|light)", None)).as_deref(),
        Ok("src/rota.rs:1:pub fn rota() -> &'static str { \"day\" }\n")
    );
    assert_eq!(
        read(
            &mut part,
            &w,
            search("pub fn (rota|light)", Some(At::Issue(watch_12())))
        )
        .as_deref(),
        Ok(
            "src/lamp.rs:1:pub fn light() {}\nsrc/rota.rs:1:pub fn rota() -> &'static str { \"night\" }\n"
        )
    );
    assert_eq!(
        read(
            &mut part,
            &w,
            search("Night", Some(At::Commit(repo.first.clone())))
        )
        .as_deref(),
        Ok("README.md:1:The Night Watch\n")
    );
    assert_eq!(
        read(&mut part, &w, search("Night", None)).as_deref(),
        Ok("no line matches Night in watch (ankh-morpork/watch) at the default branch's head\n")
    );
    let bad = read(&mut part, &w, search("(", None)).expect_err("refused");
    assert!(
        bad.starts_with("search for ( in watch (ankh-morpork/watch):"),
        "{bad}"
    );
}

#[test]
fn search_prints_at_most_its_limit_of_lines_and_counts_the_rest() {
    let dir = scratch("search-limit");
    let origin = Origin::new(&dir, "watch");
    let many: String = (0..temet_work_codehost::read::MOST_LINES + 5)
        .map(|n| format!("constable {n}\n"))
        .collect();
    origin.commit(None, &[("roll.txt", &many)], "The roll", "refs/heads/main");
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &origin)]);
    let text = read(&mut part, &w, search("constable", None)).expect("found");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), temet_work_codehost::read::MOST_LINES + 1);
    assert_eq!(lines[0], "roll.txt:1:constable 0");
    assert_eq!(
        lines.last().copied(),
        Some("and 5 more matching lines not printed; give a narrower pattern")
    );
}

#[test]
fn change_prints_the_pull_request_from_where_its_branch_left_the_default_branch() {
    let dir = scratch("change");
    let repo = watch(&dir);
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    let change = |by_commit| Command::Change {
        issue: watch_12(),
        project: None,
        by_commit,
    };
    let whole = read(&mut part, &w, change(false)).expect("change");
    let head = format!(
        "watch (ankh-morpork/watch): pull request 57, 2 commits from {} on the default branch to \
         {}; the whole change:\n",
        &repo.first[..12],
        &repo.night[..12]
    );
    assert!(whole.starts_with(&head), "{whole}");
    assert!(
        whole.contains("+pub fn rota() -> &'static str { \"night\" }"),
        "{whole}"
    );
    assert!(whole.contains("+++ b/src/lamp.rs"), "{whole}");
    // main moved on after the branch left it; that is no part of the change.
    assert!(!whole.contains("City Watch"), "{whole}");
    assert!(
        !whole.contains("A lamp for the night"),
        "no commit messages: {whole}"
    );

    let each = read(&mut part, &w, change(true)).expect("by commit");
    let rota = each
        .find("A night shift on the rota")
        .expect("first commit");
    let lamp = each.find("A lamp for the night").expect("second commit");
    assert!(rota < lamp, "in order: {each}");
    assert!(each.contains(&format!("commit {}\n", repo.night)), "{each}");
    assert!(each.contains("; each commit:\n"), "{each}");
    assert!(each.contains("+++ b/src/lamp.rs"), "{each}");
}

#[test]
fn change_prints_each_repository_under_its_name_and_a_project_picks_one() {
    let dir = scratch("change-two");
    let watch = watch(&dir);
    let guild = Origin::new(&dir, "guild");
    let charter = guild.commit(
        None,
        &[("charter.md", "The Guild\n")],
        "Charter",
        "refs/heads/main",
    );
    guild.commit(
        Some(&charter),
        &[("charter.md", "The Guild of Watchmen\n")],
        "Rename the guild",
        "refs/pull/57/head",
    );
    let w = city();
    let mut part = family(
        &dir,
        &[
            ("ankh-morpork/watch", &watch.origin),
            ("ankh-morpork/guild", &guild),
        ],
    );
    let both = read(
        &mut part,
        &w,
        Command::Change {
            issue: city_12(),
            project: None,
            by_commit: false,
        },
    )
    .expect("both");
    let at_watch = both
        .find("watch (ankh-morpork/watch): pull request 57")
        .expect("watch");
    let at_guild = both
        .find("guild (ankh-morpork/guild): pull request 57")
        .expect("guild");
    assert!(at_watch < at_guild, "in the plan's order: {both}");
    let one = read(
        &mut part,
        &w,
        Command::Change {
            issue: city_12(),
            project: Some(ProjectId::new("guild")),
            by_commit: false,
        },
    )
    .expect("guild");
    assert!(one.starts_with("guild (ankh-morpork/guild)"), "{one}");
    assert!(!one.contains("watch (ankh-morpork/watch)"), "{one}");
    let why = read(
        &mut part,
        &w,
        Command::Change {
            issue: city_12(),
            project: Some(ProjectId::new("lancre")),
            by_commit: false,
        },
    )
    .expect_err("refused");
    assert_eq!(
        why,
        "city#12 has no open pull request in lancre; it has one in: watch, guild"
    );
}

#[test]
fn change_refuses_an_issue_with_no_open_pull_request() {
    let dir = scratch("change-none");
    let repo = watch(&dir);
    let mut w = world(&[("watch", "ankh-morpork/watch")]);
    let mut part = family(&dir, &[("ankh-morpork/watch", &repo.origin)]);
    w.issues
        .get_mut(&watch_12())
        .expect("issue")
        .merged
        .insert(ProjectId::new("watch"));
    let why = read(
        &mut part,
        &w,
        Command::Change {
            issue: watch_12(),
            project: None,
            by_commit: false,
        },
    )
    .expect_err("refused");
    assert_eq!(
        why,
        "watch#12 has no open pull request on record; its author names one with ready"
    );
}

#[test]
fn a_family_with_no_folder_for_copies_says_so() {
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut host = FakeCodeHost::default();
    host.clones
        .insert("ankh-morpork/watch".into(), "/nowhere/watch.git".into());
    let mut part = CodeHosts::default();
    part.add(Box::new(host));
    let why = read(&mut part, &w, show("watch", "README.md", None)).expect_err("refused");
    assert!(
        why.starts_with("this daemon keeps no copies of repositories"),
        "{why}"
    );
}

#[test]
fn ci_prints_each_check_and_the_output_of_each_failing_one() {
    let w = world(&[("watch", "ankh-morpork/watch")]);
    let mut host = FakeCodeHost::default();
    host.pulls.push((
        "ankh-morpork/watch".into(),
        Pull {
            number: "57".into(),
            state: PullState::Open,
            checks: Checks::Failed {
                failing: vec!["rust".into(), "clacks".into()],
            },
            conflicting: false,
            draft: false,
            branch: "watch-12-night-shift".into(),
        },
    ));
    let run =
        |name: &str, state, log: Option<&str>, summary: &str, address: Option<&str>| CheckRun {
            name: name.into(),
            state,
            log: log.map(String::from),
            summary: summary.into(),
            address: address.map(String::from),
        };
    host.runs.push((
        "ankh-morpork/watch".into(),
        "57".into(),
        vec![
            run("docs", RunState::Passed, None, "", None),
            run(
                "rust",
                RunState::Failed,
                Some("test rota::night ... FAILED\nassertion failed: rota() == \"night\"\n"),
                "",
                Some("https://fake.example.com/ankh-morpork/watch/runs/1"),
            ),
            run(
                "clacks",
                RunState::Failed,
                None,
                "The tower at Sto Lat did not answer",
                Some("https://clacks.example.com/status/57"),
            ),
            run("deploy", RunState::Running, None, "", None),
        ],
    ));
    let mut part = CodeHosts::default();
    part.add(Box::new(host));
    let text = read(&mut part, &w, Command::Ci { issue: watch_12() }).expect("ci");
    assert_eq!(
        text,
        "watch (ankh-morpork/watch): pull request 57\n  docs: passed\n  rust: failed\n  \
         clacks: failed\n  deploy: running\n\nThe output of rust's failed steps:\ntest \
         rota::night ... FAILED\nassertion failed: rota() == \"night\"\n\nclacks keeps no log \
         the code host gives: The tower at Sto Lat did not answer\n\
         https://clacks.example.com/status/57\n"
    );
    // A code host that cannot read the checks is refused with its reason.
    let mut empty = CodeHosts::default();
    empty.add(Box::new(FakeCodeHost::default()));
    let why = read(&mut empty, &w, Command::Ci { issue: watch_12() }).expect_err("refused");
    assert!(
        why.contains("not found: ankh-morpork/watch pull request 57"),
        "{why}"
    );
}
