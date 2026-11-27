//! What the daemon answers a [`Request`](crate::Request) with.

use serde::{Deserialize, Serialize};

use crate::view::View;

/// The daemon's answer to one request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    /// Done; the text says what changed, holds the state for `status`, or is
    /// the build the daemon was asked for.
    Done {
        /// What to show.
        text: String,
    },
    /// Refused, and why.
    Refused {
        /// Why, in a line.
        why: String,
    },
    /// The state as data, for the `view` command.
    View {
        /// The state.
        view: View,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_and_refused_keep_their_lines() {
        for (reply, line) in [
            (
                Reply::Done {
                    text: "watch#1 is ready".into(),
                },
                r#"{"reply":"done","text":"watch#1 is ready"}"#,
            ),
            (
                Reply::Refused {
                    why: "there is no issue watch#2".into(),
                },
                r#"{"reply":"refused","why":"there is no issue watch#2"}"#,
            ),
        ] {
            assert_eq!(serde_json::to_string(&reply).expect("json"), line);
            assert_eq!(serde_json::from_str::<Reply>(line).expect("a reply"), reply);
        }
    }

    #[test]
    fn a_view_reply_reads_back() {
        let (world, _) = crate::World::replay(&[crate::Event::ProjectAdded {
            project: crate::ids::ProjectId::new("watch"),
            tracker: "fake".into(),
            name: "ankh-morpork/watch".into(),
            code_host: None,
            repo: None,
        }]);
        let problems = [("tracker".to_string(), "the clacks are down".to_string())].into();
        let reply = Reply::View {
            view: View::new(&world, &problems),
        };
        let line = serde_json::to_string(&reply).expect("json");
        assert!(
            line.starts_with(r#"{"reply":"view","view":{"projects":[{"id":"watch","#),
            "{line}"
        );
        assert_eq!(
            serde_json::from_str::<Reply>(&line).expect("a reply"),
            reply
        );
    }
}
