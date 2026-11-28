//! One smoke per mix of parts: `parts()` returns exactly the parts this build
//! compiled in, each once, in the registration order. scripts/check-mixes runs
//! it in every mix that matters.

#[test]
fn the_build_returns_each_compiled_in_part_once() {
    let ids: Vec<&str> = temet_work::parts().iter().map(|p| p.id()).collect();
    #[allow(unused_mut)]
    let mut want: Vec<&str> = Vec::new();
    if cfg!(feature = "tracker") {
        want.push("tracker");
    }
    if cfg!(feature = "runner") {
        want.push("runner");
    }
    if cfg!(feature = "runtime") {
        want.push("runtime");
    }
    if cfg!(feature = "codehost") {
        want.push("codehost");
    }
    if cfg!(feature = "deploy") {
        want.push("deploy");
    }
    if cfg!(feature = "checkout") {
        want.push("checkout");
    }
    if cfg!(feature = "store") {
        want.push("store");
    }
    if cfg!(feature = "style") {
        want.push("style");
    }
    if cfg!(feature = "mike") {
        want.push("mike");
    }
    assert_eq!(ids, want);
}

/// A build with no tracker part says, in the issue's own view, that nothing in it
/// reads issue text; one with a tracker says why that one has not.
#[test]
fn the_issue_view_says_why_its_text_was_not_read() {
    use temet_work_core::ids::IssueId;
    use temet_work_core::{Command, Event, Host, Peer, Request, World};

    let memory = Box::new(temet_work_core::host::Memory::default());
    let mut host = Host::new(World::default(), temet_work::parts(), memory);
    let issue = IssueId::new("ankh", "1");
    let synced = Event::IssueSynced {
        issue: issue.clone(),
        title: "Mend the clock".into(),
        milestone: None,
        depends_on: vec![],
        order: 1,
        closed: false,
    };
    host.handle(0, synced).expect("synced");
    let request = Request::new(None, Command::Issue { issue });
    let reply = temet_work::socket::answer(&mut host, 1, &request, &Peer::Unchecked);
    let Ok(temet_work::socket::Reply::Done { text }) = reply else {
        panic!("{reply:?}");
    };
    let why = if cfg!(feature = "tracker") {
        "ankh has no tracker configured"
    } else {
        "no part in this build reads issue text"
    };
    let line = format!("\nThe issue's text was not read: {why}");
    assert!(text.contains(&line), "{text}");
}
