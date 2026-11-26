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
