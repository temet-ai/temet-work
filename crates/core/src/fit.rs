//! Holding a brief, and a message to a running agent, to one size.
//!
//! A brief is one argument of the command that starts its agent, and Linux
//! refuses one over 128 KiB; a next-round or fix message goes into a running
//! agent's context. Each is held to [`LIMIT`]. A check, rule, point or reply is
//! printed with at most [`ITEM_CHARS`] of its characters, followed by a pointer
//! to the command that prints it whole. The parts of a text share the room by
//! [`Rank`], so what its reader acts on comes first, and a line counts what is
//! left out and names the command that prints it.

use std::ops::Range;

use crate::text::lined;

/// The most bytes a finished brief, next-round message or fix message holds:
/// 28 KiB under the 128 KiB that Linux allows one argument.
pub(crate) const LIMIT: usize = 100 * 1024;

/// The most characters of one check, rule, point or reply that a brief or a
/// message prints.
pub(crate) const ITEM_CHARS: usize = 1000;

/// The fewest bytes a text of a list that shrinks is cut to, pointer and all;
/// below it, entries are left out instead.
const LEAST_TEXT: usize = 200;

/// The most of a cut text, such as an arbiter's latest review round, that the
/// ranks before it keep back for it: they could otherwise leave it no room.
const LEAST_CUT: usize = 16 * 1024;

/// Said where the last guard cut a part of plain text.
const GUARD_NOTE: &str = "\n[The rest of this part is left out to hold the text to 100 KiB.]\n";

/// The order parts are given room in: an earlier rank takes what it needs of
/// the room left, keeping back only what each later part is owed
/// ([`Part::least`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Rank {
    /// The checks, points and replies its reader judges or answers.
    ActedOn,
    /// An arbiter's latest review round.
    LatestRound,
    /// The kept rules.
    Rules,
    /// An arbiter's earlier review rounds.
    EarlierRounds,
}

/// One entry of a list as printed, given the most bytes each of its texts may
/// take.
pub(crate) type Entry = Box<dyn Fn(usize) -> String>;

/// The line saying which entries of a list are left out, given their places.
type LeftOut = Box<dyn Fn(Range<usize>) -> String>;

/// Entries that are cut, and then left out, to fit the room their rank leaves.
pub(crate) struct List {
    entries: Vec<Entry>,
    left_out: LeftOut,
    /// Whether the latest entries are kept, with the line counting the earlier
    /// ones first; else the first are kept, and the line ends the list.
    latest: bool,
    /// Whether each text is cut shorter than [`ITEM_CHARS`] characters, down to
    /// [`LEAST_TEXT`] bytes, before any entry is left out.
    shrinks: bool,
}

/// One part of a text, in the order it is printed.
pub(crate) enum Part {
    /// Printed whole, unless the parts that fit leave the text over [`LIMIT`].
    Whole(String),
    /// The command its reader reports with: never cut.
    Command(String),
    /// A list, given room by its rank.
    List(Rank, List),
    /// A text cut at the room its rank leaves, followed by the line given, which
    /// says so.
    Cut(Rank, String, String),
}

impl Part {
    /// Entries its reader acts on. Each text is cut shorter before any entry is
    /// left out, and the last are left out first.
    pub(crate) fn acted_on(
        entries: Vec<Entry>,
        left_out: impl Fn(Range<usize>) -> String + 'static,
    ) -> Self {
        Self::List(
            Rank::ActedOn,
            List {
                entries,
                left_out: Box::new(left_out),
                latest: false,
                shrinks: true,
            },
        )
    }

    /// Entries at `rank` of which the latest that fit are kept, each with its
    /// texts at most [`ITEM_CHARS`] characters.
    pub(crate) fn latest(
        rank: Rank,
        entries: Vec<Entry>,
        left_out: impl Fn(Range<usize>) -> String + 'static,
    ) -> Self {
        Self::List(
            rank,
            List {
                entries,
                left_out: Box::new(left_out),
                latest: true,
                shrinks: false,
            },
        )
    }

    fn rank(&self) -> Option<Rank> {
        match self {
            Self::Whole(_) | Self::Command(_) => None,
            Self::List(rank, _) | Self::Cut(rank, ..) => Some(*rank),
        }
    }

    /// The bytes the ranks before it keep back for it: what a list prints with
    /// every entry left out, and up to [`LEAST_CUT`] of a cut text.
    fn least(&self) -> usize {
        match self {
            Self::Whole(_) | Self::Command(_) => 0,
            Self::List(_, l) if l.entries.is_empty() => 0,
            Self::List(_, l) => (l.left_out)(0..l.entries.len()).len(),
            Self::Cut(_, text, _) if text.len() <= LEAST_CUT => text.len(),
            Self::Cut(_, _, line) => LEAST_CUT + 1 + line.len(),
        }
    }
}

/// A check, rule, point or reply as a brief prints it: cleaned as
/// [`crate::text::clean`] cleans it, and no more than its first
/// [`ITEM_CHARS`] characters. A list prints one many times while it looks for
/// the room, so it is made once.
pub(crate) struct Item {
    /// Its first [`ITEM_CHARS`] characters, cleaned.
    text: String,
    /// Whether it has more.
    longer: bool,
    /// How many line breaks `text` holds.
    breaks: usize,
}

impl Item {
    pub(crate) fn new(raw: &str) -> Self {
        let mut chars = crate::text::cleaned(raw);
        let text: String = chars.by_ref().take(ITEM_CHARS).collect();
        let longer = chars.next().is_some();
        Self {
            breaks: text.matches('\n').count(),
            text,
            longer,
        }
    }

    /// The item with each later line indented `by` spaces: whole when it has
    /// at most [`ITEM_CHARS`] characters and takes at most `room` bytes; else
    /// its first characters, at most [`ITEM_CHARS`] of them and never part of
    /// one, that leave room for `pointer`, followed by it.
    pub(crate) fn printed(&self, by: usize, room: usize, pointer: &str) -> String {
        let size = self.text.len() + by * self.breaks;
        if !self.longer && size <= room {
            return lined(&self.text, by);
        }
        let room = room.saturating_sub(pointer.len());
        if size <= room {
            return format!("{}{pointer}", lined(&self.text, by));
        }
        // The longest start whose lines, indented, take at most `room` bytes.
        let mut left = room;
        let mut end = 0;
        for (n, line) in self.text.split('\n').enumerate() {
            if n > 0 {
                if left < 1 + by {
                    break;
                }
                left -= 1 + by;
                end += 1;
            }
            if line.len() > left {
                end += line.floor_char_boundary(left);
                break;
            }
            left -= line.len();
            end += line.len();
        }
        format!("{}{pointer}", lined(&self.text[..end], by))
    }
}

/// The text of `parts` in their order, at most [`LIMIT`] bytes while the
/// commands and the least the other parts print fit in it.
///
/// The plain parts and commands are printed first. The room left goes to the
/// other parts rank by rank, keeping back what each later part is owed
/// ([`Part::least`]). A part of plain text is cut only when the parts that fit
/// still leave the text over [`LIMIT`]: the longest first, with a line saying
/// so.
pub(crate) fn laid_out(parts: Vec<Part>) -> String {
    let mut texts: Vec<Option<String>> = parts
        .iter()
        .map(|p| match p {
            Part::Whole(t) | Part::Command(t) => Some(t.clone()),
            Part::List(..) | Part::Cut(..) => None,
        })
        .collect();
    let fixed: usize = texts.iter().flatten().map(String::len).sum();
    let mut left = LIMIT.saturating_sub(fixed);
    let mut ranks: Vec<Rank> = parts.iter().filter_map(Part::rank).collect();
    ranks.sort();
    ranks.dedup();
    for rank in ranks {
        let later: usize = parts
            .iter()
            .filter(|p| p.rank().is_some_and(|r| r > rank))
            .map(Part::least)
            .sum();
        let room = left.saturating_sub(later);
        let here: Vec<usize> = (0..parts.len())
            .filter(|&i| parts[i].rank() == Some(rank))
            .collect();
        let lists: Vec<&List> = here
            .iter()
            .filter_map(|&i| match &parts[i] {
                Part::List(_, l) => Some(l),
                _ => None,
            })
            .collect();
        let mut fitted = fit(&lists, room).into_iter();
        let mut used = 0;
        for &i in &here {
            if let Part::List(..) = parts[i] {
                let text = fitted.next().unwrap_or_default();
                used += text.len();
                texts[i] = Some(text);
            }
        }
        for &i in &here {
            if let Part::Cut(_, text, line) = &parts[i] {
                let text = cut_at(text, line, room.saturating_sub(used));
                used += text.len();
                texts[i] = Some(text);
            }
        }
        left = left.saturating_sub(used);
    }
    guard(&parts, &mut texts);
    texts.into_iter().flatten().collect()
}

/// `text` when it fits in `room`; else as much of it as leaves room for `line`,
/// ended on a whole character, then `line`.
fn cut_at(text: &str, line: &str, room: usize) -> String {
    if text.len() <= room {
        return text.to_string();
    }
    let keep = text.floor_char_boundary(room.saturating_sub(line.len() + 1));
    let mut out = text[..keep].to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out
}

/// Cuts the longest parts of plain text in `texts` while it is over [`LIMIT`].
fn guard(parts: &[Part], texts: &mut [Option<String>]) {
    let mut total: usize = texts.iter().flatten().map(String::len).sum();
    let mut plain: Vec<usize> = (0..parts.len())
        .filter(|&i| matches!(parts[i], Part::Whole(_)))
        .collect();
    plain.sort_by_key(|&i| texts[i].as_ref().map_or(0, String::len));
    while total > LIMIT {
        let Some(i) = plain.pop() else { break };
        let Some(text) = texts[i].as_ref() else {
            continue;
        };
        let keep = text.len().saturating_sub(total - LIMIT + GUARD_NOTE.len());
        let kept = format!("{}{GUARD_NOTE}", &text[..text.floor_char_boundary(keep)]);
        if kept.len() < text.len() {
            total = total - text.len() + kept.len();
            texts[i] = Some(kept);
        }
    }
}

/// `lists`, which share one rank, as printed in `room` bytes when they fit in
/// it: whole; else, for those that shrink, with each text cut to the most bytes
/// that fit, but not below [`LEAST_TEXT`]; else with entries left out of the
/// list that keeps the most, one at a time.
fn fit(lists: &[&List], room: usize) -> Vec<String> {
    let all: Vec<usize> = lists.iter().map(|l| l.entries.len()).collect();
    let full = render(lists, usize::MAX, &all);
    if size(&full) <= room {
        return full;
    }
    if lists.iter().any(|l| l.shrinks) && size(&render(lists, LEAST_TEXT, &all)) <= room {
        let mut low = LEAST_TEXT;
        let mut high = lists
            .iter()
            .flat_map(|l| l.entries.iter().map(|e| e(usize::MAX).len()))
            .max()
            .unwrap_or(LEAST_TEXT);
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if size(&render(lists, mid, &all)) <= room {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        return render(lists, low, &all);
    }
    let lens: Vec<Vec<usize>> = lists
        .iter()
        .map(|l| {
            let cap = if l.shrinks { LEAST_TEXT } else { usize::MAX };
            l.entries.iter().map(|e| e(cap).len()).collect()
        })
        .collect();
    let line = |l: &List, kept: usize| {
        let n = l.entries.len();
        match (kept < n, l.latest) {
            (false, _) => 0,
            (true, true) => (l.left_out)(0..n - kept).len(),
            (true, false) => (l.left_out)(kept..n).len(),
        }
    };
    let mut kept = all;
    let mut shown: Vec<usize> = lens.iter().map(|l| l.iter().sum()).collect();
    let mut lines = vec![0; lists.len()];
    while shown.iter().chain(&lines).sum::<usize>() > room {
        let Some(i) = (0..kept.len())
            .filter(|&i| kept[i] > 0)
            .max_by_key(|&i| (kept[i], i))
        else {
            break;
        };
        let n = lists[i].entries.len();
        let dropped = if lists[i].latest {
            n - kept[i]
        } else {
            kept[i] - 1
        };
        shown[i] -= lens[i][dropped];
        kept[i] -= 1;
        lines[i] = line(lists[i], kept[i]);
    }
    render(lists, LEAST_TEXT, &kept)
}

/// Each of `lists` with `kept` of its entries, each text cut to `cap` bytes in a
/// list that shrinks, and the line counting the entries left out.
fn render(lists: &[&List], cap: usize, kept: &[usize]) -> Vec<String> {
    lists
        .iter()
        .zip(kept)
        .map(|(l, &k)| {
            let cap = if l.shrinks { cap } else { usize::MAX };
            let n = l.entries.len();
            let mut out = String::new();
            if l.latest {
                if k < n {
                    out.push_str(&(l.left_out)(0..n - k));
                }
                for e in &l.entries[n - k..] {
                    out.push_str(&e(cap));
                }
            } else {
                for e in &l.entries[..k] {
                    out.push_str(&e(cap));
                }
                if k < n {
                    out.push_str(&(l.left_out)(k..n));
                }
            }
            out
        })
        .collect()
}

fn size(texts: &[String]) -> usize {
    texts.iter().map(String::len).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_cut_between_characters_at_the_character_limit_or_the_room() {
        let pointer = " (cut here)";
        assert_eq!(Item::new("the lamp").printed(2, 100, pointer), "the lamp");
        assert_eq!(
            Item::new("the lamp\nis lit").printed(2, 100, pointer),
            "the lamp\n  is lit"
        );
        // Over the limit in characters: its first ITEM_CHARS, whatever their size.
        let long = "✓".repeat(ITEM_CHARS + 1);
        let shown = Item::new(&long).printed(2, usize::MAX, pointer);
        assert_eq!(shown, format!("{}{pointer}", "✓".repeat(ITEM_CHARS)));
        assert_eq!(
            Item::new(&long[3..]).printed(2, usize::MAX, pointer),
            long[3..]
        );
        // Over the room: cut to leave room for the pointer, on a whole character.
        let shown = Item::new(&"€".repeat(10)).printed(2, 9 + pointer.len(), pointer);
        assert_eq!(shown, format!("€€€{pointer}"));
        let shown = Item::new(&"€".repeat(10)).printed(2, 8 + pointer.len(), pointer);
        assert_eq!(shown, format!("€€{pointer}"));
        // An indent counts against the room.
        assert_eq!(Item::new("ab\ncd\nef").printed(4, 9, "~"), "ab\n    c~");
    }

    #[test]
    fn the_room_goes_rank_by_rank_and_the_guard_keeps_the_command() {
        let entry = |text: String| -> Entry {
            let item = Item::new(&text);
            Box::new(move |cap| item.printed(2, cap, "~") + "\n")
        };
        let many = |n: usize| {
            (0..n)
                .map(|i| entry(format!("{i:04}{}", "x".repeat(600))))
                .collect()
        };
        let text = laid_out(vec![
            Part::Whole("head\n".into()),
            Part::latest(Rank::Rules, many(100), |r| format!("{} early\n", r.len())),
            Part::acted_on(many(100), |r| format!("{} late\n", r.len())),
            Part::Command("run it\n".into()),
        ]);
        assert!(text.len() <= LIMIT, "{}", text.len());
        assert!(text.ends_with("run it\n"));
        // The entries acted on take what they need first, whole.
        assert!(text.contains(&format!("0099{}\n", "x".repeat(600))));
        assert!(!text.contains(" late\n"));
        // The latest rules fit in the rest, after the line counting the others.
        let (_, rules) = text.split_once("head\n").expect("head");
        let (count, rules) = rules.split_once('\n').expect("the count");
        assert!(count.ends_with(" early"), "{count}");
        assert!(rules.starts_with("00"), "{rules:.40}");
        assert!(rules.contains(&format!("0099{}\n0000", "x".repeat(600))));

        let huge = "y".repeat(2 * LIMIT);
        let text = laid_out(vec![
            Part::Whole(huge),
            Part::acted_on(many(10), |r| format!("{} late\n", r.len())),
            Part::Command("run it\n".into()),
        ]);
        assert!(text.len() <= LIMIT, "{}", text.len());
        assert!(text.ends_with(&format!("{GUARD_NOTE}10 late\nrun it\n")));
    }
}
