//! MeterList (`temet-meterlist`): a titled group of meters in a side column, such as every
//! machine's build slots or every account's limits.

use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Window, div};

use crate::components::Class;
use crate::components::heading::Heading;
use crate::components::meter::Meter;

/// One meter: what is measured, how full it is from 0 to 1, the figure, and a muted note.
#[derive(Clone, Debug, PartialEq)]
pub struct MeterListItem {
    pub id: SharedString,
    pub label: SharedString,
    pub value: f32,
    pub reading: SharedString,
    pub note: Option<SharedString>,
}

#[derive(IntoElement)]
pub struct MeterList {
    title: SharedString,
    meters: Vec<MeterListItem>,
}

impl MeterList {
    /// A list headed `title`, such as Machines or Limits.
    pub fn new(title: impl Into<SharedString>, meters: Vec<MeterListItem>) -> Self {
        Self {
            title: title.into(),
            meters,
        }
    }
}

impl RenderOnce for MeterList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let meters = self.meters.into_iter().map(|m| {
            let meter = Meter::new(m.label, m.value, m.reading);
            match m.note {
                Some(note) => meter.note(note),
                None => meter,
            }
        });
        div()
            .child(Heading::new(self.title).class_name(Class::ListTitle))
            .children(meters)
    }
}
