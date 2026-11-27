//! Who sent a request, and whether the daemon takes the sender at its word.
//!
//! Every agent the daemon starts gets a random secret of its own in
//! [`SECRET_VAR`](crate::launch::SECRET_VAR); the event log keeps only the
//! secret's SHA-256 digest, in [`Event::AgentStarted`](crate::Event). A request
//! that names an agent must carry that agent's secret. A request that names no
//! agent is a person's, unless the daemon can tell it came from an agent: it
//! carries an agent's secret, the daemon found one in the sending process or its
//! parents, or it came from another machine through the relay.
//!
//! The core only compares digests. Making a secret, hashing one and reading
//! another process are I/O: the part that starts agents and the daemon do them.

use serde::{Deserialize, Serialize};

use crate::command::Command;
use crate::ids::AgentId;
use crate::model::Agent;
use crate::world::World;

/// The SHA-256 digest of an agent's secret, as 64 lowercase hex digits. It can
/// be logged and shown: the secret cannot be had back from it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Digest(pub String);

impl Digest {
    /// The digest from the hash's 32 bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        use std::fmt::Write;
        let mut hex = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            let _ = write!(hex, "{b:02x}");
        }
        Self(hex)
    }
}

/// What the daemon learned about the process that sent a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Peer {
    /// The sending process and its parents were read: the digests of the agent
    /// secrets found in their environments, from any daemon.
    Read(Vec<Digest>),
    /// The sending process itself could not be read; why.
    Unreadable(String),
    /// The request came through `temet-work relay`, from another machine whose
    /// processes the daemon cannot see.
    Relayed,
    /// Not looked at: the platform has no process tree to read (macOS), or the
    /// request was made inside the daemon.
    Unchecked,
}

/// A request's sender, as far as the daemon can tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sender {
    /// The agent the request names; `None` for a person.
    pub by: Option<AgentId>,
    /// The digest of the secret the request carried.
    pub proof: Option<Digest>,
    /// What the daemon learned about the sending process.
    pub peer: Peer,
}

impl World {
    /// The agent this daemon started with the secret whose digest is `digest`.
    fn holder(&self, digest: &Digest) -> Option<&Agent> {
        self.agents
            .values()
            .find(|a| a.secret_digest.as_ref() == Some(digest))
    }

    /// Why the daemon does not take `sender` at its word for `command`.
    ///
    /// A request that names an agent needs that agent's secret, for every
    /// command, reads included; an agent started before secrets existed (no
    /// digest on record) is taken by its id. Either way, a secret this daemon
    /// gave another agent in the sender's process tree refuses it. A request
    /// that names no agent may read anything; one that changes something is
    /// refused when it carries an agent's secret, comes from inside an agent's
    /// process tree, from a process that cannot be read, or through the relay.
    pub fn admit(&self, sender: &Sender, command: &Command) -> Result<(), String> {
        let found: Vec<&AgentId> = match &sender.peer {
            Peer::Read(digests) => {
                let mut ids: Vec<&AgentId> = digests
                    .iter()
                    .filter_map(|d| self.holder(d).map(|a| &a.id))
                    .collect();
                ids.sort();
                ids.dedup();
                ids
            }
            _ => Vec::new(),
        };
        let carried = sender.proof.as_ref().and_then(|p| self.holder(p));
        let Some(by) = &sender.by else {
            if !command.changes() {
                return Ok(());
            }
            let name = command.name();
            if let Some(agent) = carried {
                return Err(format!(
                    "{name} names no agent but carries {}'s secret: an agent may not run a \
                     person's command",
                    agent.id
                ));
            }
            return match &sender.peer {
                Peer::Read(_) => match found.first() {
                    Some(agent) => Err(format!(
                        "{name} names no agent but comes from inside {agent}'s process tree: an \
                         agent may not run a person's command; a person runs it from a shell \
                         outside any agent's terminal"
                    )),
                    None => Ok(()),
                },
                Peer::Unreadable(why) => Err(format!(
                    "{name} names no agent, and the process that sent it cannot be read ({why}), \
                     so it cannot be told from an agent's"
                )),
                Peer::Relayed => Err(format!(
                    "{name} names no agent and came through temet-work relay from another \
                     machine, where the daemon cannot see who sent it: run it on the daemon's \
                     machine"
                )),
                Peer::Unchecked => Ok(()),
            };
        };
        let Some(agent) = self.agents.get(by) else {
            return Err(format!("{by} is not an agent this daemon started"));
        };
        match (&agent.secret_digest, &sender.proof, carried) {
            (_, _, Some(other)) if other.id != *by => {
                return Err(format!(
                    "the request names {by} but carries {}'s secret, not {by}'s",
                    other.id
                ));
            }
            (Some(_), None, _) => {
                return Err(format!(
                    "the request names {by} but carries no secret: {by}'s commands need the \
                     TEMET_WORK_AGENT_TOKEN it was started with"
                ));
            }
            (Some(digest), Some(proof), _) if digest != proof => {
                return Err(format!(
                    "the request names {by} but its secret is wrong: {by}'s commands need the \
                     TEMET_WORK_AGENT_TOKEN it was started with"
                ));
            }
            // The right secret, or an agent started before secrets, taken by its id.
            _ => {}
        }
        if let Some(other) = found.iter().find(|a| **a != by) {
            return Err(format!(
                "the request names {by} but comes from inside {other}'s process tree, which \
                 holds {other}'s secret"
            ));
        }
        Ok(())
    }
}

impl Command {
    /// The command's name, as the command line spells it.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Issue { .. } => "issue",
            Self::Screen { .. } => "screen",
            Self::Plan { .. } => "plan",
            Self::Ready { .. } => "ready",
            Self::Review { .. } => "review",
            Self::Set { .. } => "set",
            Self::Unset { .. } => "unset",
            Self::Settings { .. } => "settings",
            Self::Retry { .. } => "retry",
            Self::Rule { .. } => "rule",
            Self::Overturn { .. } => "overturn",
            Self::Proposal { .. } => "proposal",
        }
    }

    /// Whether the command changes anything; `status`, `issue`, `screen` and
    /// `settings` only read.
    pub fn changes(&self) -> bool {
        !matches!(
            self,
            Self::Status | Self::Issue { .. } | Self::Screen { .. } | Self::Settings { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, Verdict};
    use crate::ids::{GroupId, IssueId, MachineId, RuleId};
    use crate::model::Role;

    fn d(s: &str) -> Digest {
        Digest(s.repeat(64 / s.len()))
    }

    fn started(id: &str, role: Role, digest: Option<Digest>) -> Event {
        Event::AgentStarted {
            agent: AgentId::new(id),
            group: GroupId::new("watch"),
            role,
            issue: Some(IssueId::new("ankh", "1")),
            machine: MachineId::new("morpork"),
            folder: None,
            secret_digest: digest,
        }
    }

    /// An author and a reviewer with secrets, and an arbiter started before
    /// secrets existed.
    fn world() -> World {
        World::replay(&[
            started("author-1", Role::Author, Some(d("a"))),
            started("reviewer-1", Role::Reviewer, Some(d("b"))),
            started("arbiter-1", Role::Arbiter, None),
        ])
        .0
    }

    fn sender(by: Option<&str>, proof: Option<&str>, peer: Peer) -> Sender {
        Sender {
            by: by.map(AgentId::new),
            proof: proof.map(d),
            peer,
        }
    }

    fn approve() -> Command {
        Command::Review {
            issue: IssueId::new("ankh", "1"),
            verdict: Verdict::Approve,
        }
    }

    #[test]
    fn digest_from_bytes_is_lowercase_hex() {
        assert_eq!(Digest::from_bytes(&[0, 0xab, 0x10]).0, "00ab10");
    }

    #[test]
    fn a_named_agent_needs_its_own_secret_for_every_command() {
        let w = world();
        let review = |s: Sender| w.admit(&s, &approve());
        assert_eq!(
            review(sender(Some("reviewer-1"), Some("b"), Peer::Unchecked)),
            Ok(())
        );
        let missing = review(sender(Some("reviewer-1"), None, Peer::Unchecked)).expect_err("no");
        assert!(
            missing.contains("reviewer-1") && missing.contains("no secret"),
            "{missing}"
        );
        let wrong = review(sender(Some("reviewer-1"), Some("c"), Peer::Unchecked)).expect_err("no");
        assert!(
            wrong.contains("reviewer-1") && wrong.contains("wrong"),
            "{wrong}"
        );
        let swapped =
            review(sender(Some("reviewer-1"), Some("a"), Peer::Unchecked)).expect_err("no");
        assert_eq!(
            swapped,
            "the request names reviewer-1 but carries author-1's secret, not reviewer-1's"
        );
        let unknown =
            review(sender(Some("reviewer-9"), Some("b"), Peer::Unchecked)).expect_err("no");
        assert_eq!(unknown, "reviewer-9 is not an agent this daemon started");
        // Reads too: any request that names an agent.
        let status = w.admit(
            &sender(Some("reviewer-1"), None, Peer::Unchecked),
            &Command::Status,
        );
        assert!(status.is_err());
        // Through the relay the secret is proof enough.
        assert_eq!(
            review(sender(Some("reviewer-1"), Some("b"), Peer::Relayed)),
            Ok(())
        );
    }

    #[test]
    fn an_agent_started_before_secrets_is_taken_by_its_id() {
        let w = world();
        let rule = Command::Rule {
            issue: IssueId::new("ankh", "1"),
            approve: true,
            reason: "fine".into(),
        };
        assert_eq!(
            w.admit(&sender(Some("arbiter-1"), None, Peer::Read(vec![])), &rule),
            Ok(())
        );
        // Unless the sender's own process tree holds another agent's secret.
        let why = w
            .admit(
                &sender(Some("arbiter-1"), None, Peer::Read(vec![d("a")])),
                &rule,
            )
            .expect_err("refused");
        assert!(
            why.contains("arbiter-1") && why.contains("author-1's process tree"),
            "{why}"
        );
    }

    #[test]
    fn another_agents_secret_in_the_process_tree_refuses_a_stolen_one() {
        let w = world();
        // author-1 read reviewer-1's secret and sends it as reviewer-1.
        let why = w
            .admit(
                &sender(Some("reviewer-1"), Some("b"), Peer::Read(vec![d("a")])),
                &approve(),
            )
            .expect_err("refused");
        assert_eq!(
            why,
            "the request names reviewer-1 but comes from inside author-1's process tree, \
             which holds author-1's secret"
        );
        // Its own secret in the tree, and one from another daemon, are fine.
        assert_eq!(
            w.admit(
                &sender(
                    Some("reviewer-1"),
                    Some("b"),
                    Peer::Read(vec![d("b"), d("f")])
                ),
                &approve()
            ),
            Ok(())
        );
    }

    #[test]
    fn a_person_command_from_inside_an_agent_is_refused_and_reads_still_answer() {
        let w = world();
        let keep = Command::Proposal {
            rule: RuleId(1),
            keep: true,
        };
        let inside = Peer::Read(vec![d("a")]);
        let why = w
            .admit(&sender(None, None, inside.clone()), &keep)
            .expect_err("refused");
        assert!(
            why.starts_with("proposal names no agent but comes from inside author-1's"),
            "{why}"
        );
        for read in [
            Command::Status,
            Command::Screen {
                agent: AgentId::new("author-1"),
            },
        ] {
            assert_eq!(w.admit(&sender(None, None, inside.clone()), &read), Ok(()));
        }
        // Outside any agent, and with only another daemon's secrets around.
        assert_eq!(
            w.admit(&sender(None, None, Peer::Read(vec![])), &keep),
            Ok(())
        );
        assert_eq!(
            w.admit(&sender(None, None, Peer::Read(vec![d("f")])), &keep),
            Ok(())
        );
        assert_eq!(w.admit(&sender(None, None, Peer::Unchecked), &keep), Ok(()));
    }

    #[test]
    fn a_person_command_that_cannot_be_traced_is_refused_saying_why() {
        let w = world();
        let set = Command::Set {
            target: crate::settings::Target::Global,
            role: None,
            key: "model".into(),
            value: "opus".into(),
        };
        let carried = w
            .admit(&sender(None, Some("a"), Peer::Read(vec![])), &set)
            .expect_err("refused");
        assert!(carried.contains("carries author-1's secret"), "{carried}");
        let unreadable = w
            .admit(
                &sender(None, None, Peer::Unreadable("process 7: gone".into())),
                &set,
            )
            .expect_err("refused");
        assert!(unreadable.contains("process 7: gone"), "{unreadable}");
        let relayed = w
            .admit(&sender(None, None, Peer::Relayed), &set)
            .expect_err("refused");
        assert!(
            relayed.contains("temet-work relay")
                && relayed.contains("run it on the daemon's machine"),
            "{relayed}"
        );
        assert_eq!(
            w.admit(&sender(None, None, Peer::Relayed), &Command::Status),
            Ok(())
        );
    }
}
