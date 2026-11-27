//! What an agent's screen says about the error its turn ended on: the line that
//! names it, and when the limit on that line resets. Both are pure: text, a time
//! and a time zone go in, and nothing is read from anywhere else.

use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;

/// How long after the time a line names that time still counts as passed today,
/// so a restart or a slow poll just after a reset continues at once instead of
/// waiting a day.
const PASSED_FOR_MS: i64 = 60 * 60 * 1000;

/// The line on `screen` that names the error a turn ended on: the last line
/// above the screen's bottom rule that says "error" or "limit", else the last
/// such line below it, with leading marks such as `⎿` and `⚠` taken off. A
/// status bar under the rule repeats what the conversation above already said,
/// in fewer words. `None` when no line names an error.
pub fn error_line(screen: &str) -> Option<String> {
    let lines: Vec<&str> = screen.lines().collect();
    let rule = lines.iter().rposition(|l| is_rule(l));
    let (above, below) = lines.split_at(rule.unwrap_or(lines.len()));
    above
        .iter()
        .rev()
        .chain(below.iter().rev())
        .map(|l| {
            l.trim_start_matches(|c: char| !c.is_alphanumeric())
                .trim_end()
        })
        .find(|l| {
            let l = l.to_lowercase();
            l.contains("error") || l.contains("limit")
        })
        .map(String::from)
}

/// Whether a line is a horizontal rule, as a terminal app draws one above and
/// below its input box.
fn is_rule(line: &str) -> bool {
    let line = line.trim();
    line.chars().count() >= 3 && line.chars().all(|c| c == '─' || c == '━')
}

/// When the limit `line` names resets, in milliseconds since 1970, for a line
/// seen at `seen_ms`: the clock time after "resets ", read in `zone` as the next
/// time the clock shows it, where a time up to an hour past counts as passed
/// and is returned as it is. Three forms are read: `resets 5pm`,
/// `resets 5:40pm` and `resets 17:40`; anything after the time, such as a zone
/// in brackets, is left alone. Any other line gives `None`.
pub fn reset_time(line: &str, seen_ms: u64, zone: &TimeZone) -> Option<u64> {
    let (hour, minute) = line
        .match_indices("resets ")
        .find_map(|(at, word)| clock(&line[at + word.len()..]))?;
    let seen_ms = i64::try_from(seen_ms).ok()?;
    let today = Timestamp::from_millisecond(seen_ms)
        .ok()?
        .to_zoned_datetime(zone.clone())
        .date();
    let at = |day: Date| {
        zone.to_zoned_datetime(day.at(hour, minute, 0, 0))
            .ok()
            .map(|z| z.timestamp().as_millisecond())
    };
    let mut reset = at(today)?;
    if reset < seen_ms - PASSED_FOR_MS {
        reset = at(today.tomorrow().ok()?)?;
    }
    u64::try_from(reset).ok()
}

/// Reads `5pm`, `5:40pm` or `17:40` at the start of `text`, as an hour of the
/// day and a minute. The time must end the text or be followed by something
/// other than a letter or digit.
fn clock(text: &str) -> Option<(i8, i8)> {
    let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
    let n = digits(text);
    if !(1..=2).contains(&n) {
        return None;
    }
    let hour: i8 = text[..n].parse().ok()?;
    let mut rest = &text[n..];
    let mut minute = None;
    if let Some(after) = rest.strip_prefix(':') {
        if digits(after) != 2 {
            return None;
        }
        minute = Some(after[..2].parse::<i8>().ok()?);
        rest = &after[2..];
    }
    let half = if let Some(after) = rest.strip_prefix("am") {
        rest = after;
        Some(0)
    } else if let Some(after) = rest.strip_prefix("pm") {
        rest = after;
        Some(12)
    } else {
        None
    };
    if rest.chars().next().is_some_and(char::is_alphanumeric) {
        return None;
    }
    let minute = match (minute, half) {
        (Some(m), _) if (0..60).contains(&m) => m,
        (None, Some(_)) => 0,
        // `resets 17` alone, or minutes past 59.
        _ => return None,
    };
    let hour = match half {
        Some(add) if (1..=12).contains(&hour) => hour % 12 + add,
        None if (0..24).contains(&hour) => hour,
        _ => return None,
    };
    Some((hour, minute))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 27 November 2026 at `hh:mm` UTC, in milliseconds since 1970.
    fn nov27(hh: i8, mm: i8) -> u64 {
        let ms = TimeZone::UTC
            .to_zoned_datetime(jiff::civil::date(2026, 11, 27).at(hh, mm, 0, 0))
            .expect("a time")
            .timestamp()
            .as_millisecond();
        u64::try_from(ms).expect("after 1970")
    }

    const DAY: u64 = 24 * 60 * 60 * 1000;

    #[test]
    fn reset_time_reads_the_issues_line_as_today_at_17_40() {
        let line = "You've hit your 5-hour limit · resets 5:40pm";
        assert_eq!(
            reset_time(line, nov27(16, 50), &TimeZone::UTC),
            Some(nov27(17, 40))
        );
    }

    #[test]
    fn reset_time_reads_5pm_and_17_40() {
        let seen = nov27(16, 50);
        assert_eq!(
            reset_time("limit reached · resets 5pm", seen, &TimeZone::UTC),
            Some(nov27(17, 0))
        );
        assert_eq!(
            reset_time("limit reached · resets 17:40", seen, &TimeZone::UTC),
            Some(nov27(17, 40))
        );
        assert_eq!(
            reset_time("limit reached · resets 12am", seen, &TimeZone::UTC),
            Some(nov27(0, 0) + DAY),
            "midnight"
        );
    }

    #[test]
    fn reset_time_30_minutes_past_counts_as_passed() {
        let line = "You've hit your 5-hour limit · resets 5:40pm";
        assert_eq!(
            reset_time(line, nov27(18, 10), &TimeZone::UTC),
            Some(nov27(17, 40)),
            "a restart just after the reset continues at once"
        );
    }

    #[test]
    fn reset_time_2_hours_past_means_tomorrow() {
        let line = "You've hit your 5-hour limit · resets 5:40pm";
        assert_eq!(
            reset_time(line, nov27(19, 40), &TimeZone::UTC),
            Some(nov27(17, 40) + DAY)
        );
    }

    #[test]
    fn reset_time_of_a_line_with_no_time_it_can_read_is_none() {
        let seen = nov27(16, 50);
        for line in [
            "API Error: 529 overloaded",
            "",
            "You've hit your weekly limit · resets Dec 1, 9am",
            "limit reached · resets 17",
            "limit reached · resets 5:4pm",
            "limit reached · resets 13pm",
            "limit reached · resets 24:00",
            "limit reached · resets 5:40pmish",
            "limit reached · resets in 3 hours",
        ] {
            assert_eq!(reset_time(line, seen, &TimeZone::UTC), None, "{line}");
        }
    }

    #[test]
    fn reset_time_is_read_in_the_daemons_zone_and_ignores_what_follows() {
        // 16:50 in a zone two hours ahead of UTC, as the issue saw it.
        let ahead = TimeZone::fixed(jiff::tz::Offset::constant(2));
        for line in [
            "You've hit your 5-hour limit · resets 5:40pm (Europe/Lisbon)",
            "Usage limit reached · limit resets 5:40pm · example.com/wrap-up · /upgrade to keep using Claude Code",
        ] {
            assert_eq!(
                reset_time(line, nov27(14, 50), &ahead),
                Some(nov27(15, 40)),
                "{line}"
            );
        }
    }

    /// The bottom of a terminal whose turn ended on a usage limit, as herdr-core
    /// read it, with its link and time zone swapped for made-up ones.
    const LIMITED: &str = "\
  ⎿  You've hit your 5-hour limit · resets 5:40pm (Europe/Lisbon)
     /upgrade or /usage-credits to finish what you’re working on.
─────────────────────────────────────────────────────
❯
─────────────────────────────────────────────────────
  ⚠ Usage limit reached · limit resets 5:40pm · example.com/wrap-up · /upgrade to keep using Claude Code
  ⏵⏵ auto mode on (shift+tab to cycle) · ← 1 agent";

    #[test]
    fn the_error_line_is_the_last_one_above_the_bottom_rule_that_names_it() {
        assert_eq!(
            error_line(LIMITED).as_deref(),
            Some("You've hit your 5-hour limit · resets 5:40pm (Europe/Lisbon)")
        );
    }

    #[test]
    fn the_status_bar_names_the_error_when_nothing_above_does() {
        let bar_only = LIMITED.lines().skip(2).collect::<Vec<_>>().join("\n");
        assert_eq!(
            error_line(&bar_only).as_deref(),
            Some(
                "Usage limit reached · limit resets 5:40pm · example.com/wrap-up · /upgrade to keep \
                 using Claude Code"
            )
        );
        assert_eq!(
            error_line("● API Error: 529 overloaded\n> ").as_deref(),
            Some("API Error: 529 overloaded")
        );
        assert_eq!(error_line("$ cat\nall is well\n"), None);
    }
}
