//! StageCounts (`temet-stagecounts`): a stand-in with the agreed API until the part lands from
//! the board branch.

use gpui::{App, IntoElement, ParentElement, RenderOnce, Window, div};

use crate::components::stage_tag::Stage;

#[derive(IntoElement)]
pub struct StageCounts {
    counts: Vec<(Stage, usize)>,
}

impl StageCounts {
    pub fn new(counts: Vec<(Stage, usize)>) -> Self {
        Self { counts }
    }
}

impl RenderOnce for StageCounts {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().children(
            self.counts
                .into_iter()
                .map(|(s, n)| format!("{} {n}", s.name())),
        )
    }
}
