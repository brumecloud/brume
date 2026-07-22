use std::{env, io::IsTerminal};

use anyhow::Result;
use brume_core::{ListPlansResponse, PlanSummary, Visibility};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

impl OutputFormat {
    pub fn is_json(self) -> bool {
        self == Self::Json
    }
}

pub fn json<T: Serialize>(value: &T) -> Result<()> {
    serde_json::to_writer(std::io::stdout().lock(), value)?;
    println!();
    Ok(())
}

pub fn plans(response: &ListPlansResponse) -> Result<()> {
    if !std::io::stdout().is_terminal() {
        print_plain_plans(&response.plans);
        return Ok(());
    }

    print!(
        "{}",
        render_plan_table(&response.plans, &TerminalStyle::detect())
    );
    Ok(())
}

fn print_plain_plans(plans: &[PlanSummary]) {
    println!("SLUG\tTITLE\tVISIBILITY\tPUBLISHED\tLAST READ\tEXPIRES\tURL");
    for plan in plans {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            plan.slug,
            single_line(&plan.title),
            plan.visibility,
            plan.published_at.to_rfc3339(),
            optional_timestamp(plan.last_read_at.as_ref(), "never"),
            optional_timestamp(plan.expires_at.as_ref(), "pinned"),
            plan.url,
        );
    }
}

fn optional_timestamp(value: Option<&DateTime<Utc>>, empty: &str) -> String {
    value
        .map(DateTime::to_rfc3339)
        .unwrap_or_else(|| empty.to_owned())
}

#[derive(Debug, Clone, Copy)]
struct TerminalStyle {
    color: bool,
    hyperlinks: bool,
}

impl TerminalStyle {
    fn detect() -> Self {
        let terminal = env::var("TERM").unwrap_or_default();
        let color = env::var_os("NO_COLOR").is_none() && terminal != "dumb";
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let hyperlinks = terminal != "dumb"
            && (matches!(
                term_program.as_str(),
                "iTerm.app" | "WezTerm" | "vscode" | "ghostty" | "Hyper" | "WarpTerminal"
            ) || env::var_os("VTE_VERSION").is_some()
                || env::var_os("WT_SESSION").is_some()
                || env::var_os("KONSOLE_VERSION").is_some());
        Self { color, hyperlinks }
    }
}

fn render_plan_table(plans: &[PlanSummary], style: &TerminalStyle) -> String {
    if plans.is_empty() {
        return format!("{}\n", styled("No plans found.", "\u{1b}[2m", style.color));
    }

    let now = Utc::now();
    let widths = [28, 10, 12, 12, 12];
    let top = border('┌', '┬', '┐', '─', &widths);
    let header_separator = border('├', '┼', '┤', '─', &widths);
    let row_separator = border('├', '┬', '┤', '─', &widths);
    let bottom = border('└', '─', '┘', '─', &[widths.iter().sum::<usize>() + 12]);
    let inner_width = widths.iter().sum::<usize>() + 12;
    let mut rendered = String::new();

    rendered.push_str(&top);
    rendered.push('\n');
    rendered.push_str(&table_row(
        &["PLAN", "ACCESS", "PUBLISHED", "LAST READ", "RETENTION"],
        &widths,
        Some("\u{1b}[1;2m"),
        style.color,
    ));
    rendered.push('\n');
    rendered.push_str(&header_separator);
    rendered.push('\n');

    for (index, plan) in plans.iter().enumerate() {
        let published = relative_time(plan.published_at, now);
        let last_read = plan
            .last_read_at
            .map(|value| relative_time(value, now))
            .unwrap_or_else(|| "never".to_owned());
        let retention = if plan.pinned {
            "pinned".to_owned()
        } else {
            plan.expires_at
                .map(|value| {
                    if value <= now {
                        "expired".to_owned()
                    } else {
                        relative_time(value, now)
                    }
                })
                .unwrap_or_else(|| "no expiry".to_owned())
        };
        let title = truncate(&single_line(&plan.title), widths[0]);
        let visibility = plan.visibility.to_string();
        let cells = [
            title.as_str(),
            visibility.as_str(),
            published.as_str(),
            last_read.as_str(),
            retention.as_str(),
        ];
        rendered.push_str(&table_row_with_visibility(
            &cells,
            &widths,
            plan.visibility,
            style.color,
        ));
        rendered.push('\n');

        let slug = styled(&plan.slug, "\u{1b}[2m", style.color);
        rendered.push_str(&full_width_row(&slug, inner_width));
        rendered.push('\n');
        let url = hyperlink(&plan.url, style.hyperlinks);
        rendered.push_str(&full_width_row(&url, inner_width));
        rendered.push('\n');

        if index + 1 < plans.len() {
            rendered.push_str(&row_separator);
            rendered.push('\n');
        }
    }

    rendered.push_str(&bottom);
    rendered.push('\n');
    rendered.push_str(&styled(
        &format!(
            "{} {}",
            plans.len(),
            if plans.len() == 1 { "plan" } else { "plans" }
        ),
        "\u{1b}[2m",
        style.color,
    ));
    rendered.push('\n');
    rendered
}

fn border(left: char, middle: char, right: char, fill: char, widths: &[usize]) -> String {
    let segments = widths
        .iter()
        .map(|width| fill.to_string().repeat(width + 2))
        .collect::<Vec<_>>()
        .join(&middle.to_string());
    format!("{left}{segments}{right}")
}

fn table_row(cells: &[&str], widths: &[usize], style: Option<&str>, color: bool) -> String {
    let cells = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| styled(&pad(cell, *width), style.unwrap_or(""), color))
        .collect::<Vec<_>>()
        .join(" │ ");
    format!("│ {cells} │")
}

fn table_row_with_visibility(
    cells: &[&str],
    widths: &[usize],
    visibility: Visibility,
    color: bool,
) -> String {
    let mut rendered = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| pad(cell, *width))
        .collect::<Vec<_>>();
    rendered[1] = styled(
        &rendered[1],
        match visibility {
            Visibility::Private => "\u{1b}[33m",
            Visibility::Unlisted => "\u{1b}[35m",
            Visibility::Public => "\u{1b}[32m",
        },
        color,
    );
    format!("│ {} │", rendered.join(" │ "))
}

fn full_width_row(content: &str, width: usize) -> String {
    let visible_length = visible_length(content);
    format!(
        "│ {content}{} │",
        " ".repeat(width.saturating_sub(visible_length))
    )
}

fn pad(value: &str, width: usize) -> String {
    format!(
        "{value}{}",
        " ".repeat(width.saturating_sub(value.chars().count()))
    )
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    value
        .chars()
        .take(width.saturating_sub(1))
        .chain(std::iter::once('…'))
        .collect()
}

fn single_line(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn styled(value: &str, prefix: &str, enabled: bool) -> String {
    if enabled && !prefix.is_empty() {
        format!("{prefix}{value}\u{1b}[0m")
    } else {
        value.to_owned()
    }
}

fn hyperlink(url: &str, enabled: bool) -> String {
    if enabled {
        format!("\u{1b}]8;;{url}\u{1b}\\{url}\u{1b}]8;;\u{1b}\\")
    } else {
        url.to_owned()
    }
}

fn visible_length(value: &str) -> usize {
    let mut length = 0;
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            match characters.next() {
                Some('[') => {
                    for character in characters.by_ref() {
                        if character.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    let mut previous_was_escape = false;
                    for character in characters.by_ref() {
                        if previous_was_escape && character == '\\' {
                            break;
                        }
                        previous_was_escape = character == '\u{1b}';
                    }
                }
                _ => {}
            }
        } else {
            length += 1;
        }
    }
    length
}

fn relative_time(value: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = (now - value).num_seconds();
    let (future, absolute) = if seconds < 0 {
        (true, seconds.saturating_abs())
    } else {
        (false, seconds)
    };
    let amount = if absolute < 60 {
        return if future {
            "in a moment".to_owned()
        } else {
            "just now".to_owned()
        };
    } else if absolute < 3_600 {
        format!("{}m", absolute / 60)
    } else if absolute < 86_400 {
        format!("{}h", absolute / 3_600)
    } else {
        format!("{}d", absolute / 86_400)
    };
    if future {
        format!("in {amount}")
    } else {
        format!("{amount} ago")
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;

    fn plan() -> PlanSummary {
        PlanSummary {
            id: Uuid::nil(),
            owner_handle: "planchon".to_owned(),
            slug: "example-plan".to_owned(),
            title: "Example plan".to_owned(),
            visibility: Visibility::Private,
            url: "https://plan.brume.dev/planchon/example-plan".to_owned(),
            published_at: Utc.with_ymd_and_hms(2026, 7, 22, 12, 0, 0).unwrap(),
            last_read_at: None,
            expires_at: Some(Utc.with_ymd_and_hms(2026, 8, 6, 12, 0, 0).unwrap()),
            pinned: false,
        }
    }

    #[test]
    fn table_contains_readable_metadata_and_url() {
        let rendered = render_plan_table(
            &[plan()],
            &TerminalStyle {
                color: false,
                hyperlinks: false,
            },
        );

        assert!(rendered.contains("PLAN"));
        assert!(rendered.contains("Example plan"));
        assert!(rendered.contains("example-plan"));
        assert!(rendered.contains("private"));
        assert!(rendered.contains("never"));
        assert!(rendered.contains("https://plan.brume.dev/planchon/example-plan"));
        assert!(rendered.contains("1 plan"));
        let table_lines = rendered.lines().take(7).collect::<Vec<_>>();
        assert!(
            table_lines
                .windows(2)
                .all(|lines| visible_length(lines[0]) == visible_length(lines[1]))
        );
    }

    #[test]
    fn hyperlink_keeps_the_url_visible_and_clickable() {
        let url = "https://plan.brume.dev/planchon/example-plan";
        let rendered = hyperlink(url, true);

        assert_eq!(
            rendered,
            format!("\u{1b}]8;;{url}\u{1b}\\{url}\u{1b}]8;;\u{1b}\\")
        );
        assert_eq!(visible_length(&rendered), url.chars().count());
    }

    #[test]
    fn relative_dates_are_compact() {
        let now = Utc.with_ymd_and_hms(2026, 7, 22, 12, 0, 0).unwrap();

        assert_eq!(
            relative_time(now - chrono::Duration::minutes(8), now),
            "8m ago"
        );
        assert_eq!(
            relative_time(now + chrono::Duration::days(15), now),
            "in 15d"
        );
    }
}
