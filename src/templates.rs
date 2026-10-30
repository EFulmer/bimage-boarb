use std::time::{SystemTime, UNIX_EPOCH};

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::models;

// Inlined into every page so serving it doesn't need its own route.
const STYLESHEET: &str = include_str!("../static/style.css");
const DEFAULT_SCREEN_NAME: &str = "Zorro";
const EXCERPT_CHARS: usize = 280;

pub fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · bimage-boarb" }
                style { (PreEscaped(STYLESHEET)) }
            }
            body {
                a.skip-link href="#main" { "Skip to content" }
                header.site-header {
                    div.wrap {
                        a.wordmark href="/boards" { "bimage" span.dot { "·" } "boarb" }
                        nav aria-label="Site" {
                            a href="/boards" { "Boards" }
                        }
                    }
                }
                main #main .wrap { (body) }
                footer.site-footer {
                    div.wrap { "Made by friends, for friends." }
                }
            }
        }
    }
}

pub fn boards_page(boards: &[models::Board]) -> Markup {
    layout(
        "Boards",
        html! {
            h1.page-title { "Boards" }
            @if boards.is_empty() {
                p.empty { "No boards yet." }
            } @else {
                ul.board-grid {
                    @for board in boards {
                        li {
                            a.board-card href={ "/boards/" (board.id) } {
                                span.slug { "/" (board.slug) "/" }
                                span.board-name { (board.name) }
                                span.board-description { (board.description) }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn board_page(board: &models::Board, threads: &[models::Thread]) -> Markup {
    layout(
        &board.name,
        html! {
            (board_header(board))
            @if threads.is_empty() {
                p.empty { "No threads yet." }
            } @else {
                ol.thread-list {
                    @for thread in threads {
                        li {
                            article.thread-summary {
                                h2 {
                                    a href={ "/boards/" (board.id) "/threads/" (thread.id) } { (thread.title) }
                                }
                                p.meta {
                                    "Started " (timestamp(thread.created_at))
                                    " · last bump " (timestamp(thread.bumped_at))
                                }
                                p.excerpt { (excerpt(&thread.body, EXCERPT_CHARS)) }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn thread_page(
    board: &models::Board,
    thread: &models::Thread,
    posts: &[models::Post],
) -> Markup {
    layout(
        &thread.title,
        html! {
            nav.breadcrumbs aria-label="Breadcrumb" {
                a href="/boards" { "Boards" }
                " / "
                a href={ "/boards/" (board.id) } { "/" (board.slug) "/" }
            }
            article.thread {
                header.op {
                    h1.page-title { (thread.title) }
                    p.meta { "Started " (timestamp(thread.created_at)) }
                }
                div.post-body { (thread.body) }
            }
            section.replies aria-label="Replies" {
                h2.replies-heading { (reply_count_label(posts.len())) }
                @for post in posts { (post_card(post)) }
            }
        },
    )
}

/// A full page for error responses; the handler still sets the HTTP status itself.
pub fn error_page(status: u16, heading: &str, message: &str) -> Markup {
    layout(
        heading,
        html! {
            section.error-page {
                p.error-status { (status) }
                h1.page-title { (heading) }
                p { (message) }
                p { a href="/boards" { "Back to the boards" } }
            }
        },
    )
}

fn board_header(board: &models::Board) -> Markup {
    html! {
        header.board-header {
            p.slug { a href={ "/boards/" (board.id) } { "/" (board.slug) "/" } }
            h1.page-title { (board.name) }
            p.board-description { (board.description) }
        }
    }
}

fn post_card(post: &models::Post) -> Markup {
    let name = post
        .screen_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(DEFAULT_SCREEN_NAME);
    html! {
        article.post id={ "p" (post.id) } {
            header.post-header {
                span.screen-name { (name) }
                (timestamp(post.created_at))
                a.post-number href={ "#p" (post.id) } title="Link to this post" { "No. " (post.id) }
            }
            @if let Some(parent) = post.in_reply_to {
                p.reply-to { a href={ "#p" (parent) } { ">>" (parent) } }
            }
            div.post-body { (post.body) }
        }
    }
}

fn reply_count_label(count: usize) -> String {
    match count {
        0 => "No replies yet".to_string(),
        1 => "1 reply".to_string(),
        n => format!("{n} replies"),
    }
}

/// Truncates on a character boundary, so multi-byte text (emoji, accents) can't split mid-character.
fn excerpt(body: &str, max_chars: usize) -> String {
    match body.char_indices().nth(max_chars) {
        Some((cut, _)) => format!("{}…", body[..cut].trim_end()),
        None => body.to_string(),
    }
}

/// Relative time as the text, with the exact UTC time as a tooltip and machine-readable attribute.
fn timestamp(unix: i64) -> Markup {
    html! {
        time datetime=(iso8601(unix)) title=(full_date(unix)) { (relative_time(unix, now())) }
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn relative_time(unix: i64, now: i64) -> String {
    let ago = now - unix;
    match ago {
        i64::MIN..=59 => "just now".to_string(),
        60..=3_599 => format!("{}m ago", ago / 60),
        3_600..=86_399 => format!("{}h ago", ago / 3_600),
        86_400..=2_591_999 => format!("{}d ago", ago / 86_400),
        _ => short_date(unix),
    }
}

fn iso8601(unix: i64) -> String {
    let (y, mo, d, h, mi, s) = utc_parts(unix);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn full_date(unix: i64) -> String {
    let (y, mo, d, h, mi, _) = utc_parts(unix);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02} UTC")
}

fn short_date(unix: i64) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (y, mo, d, ..) = utc_parts(unix);
    format!("{} {d}, {y}", MONTHS[(mo - 1) as usize])
}

/// Splits a Unix timestamp into UTC (year, month, day, hour, minute, second).
/// Uses Howard Hinnant's days-to-civil algorithm to avoid pulling in a date library.
fn utc_parts(unix: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = unix.div_euclid(86_400);
    let secs = unix.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day, secs / 3_600, secs % 3_600 / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_parts_known_dates() {
        assert_eq!(utc_parts(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(utc_parts(951_782_400), (2000, 2, 29, 0, 0, 0));
        assert_eq!(utc_parts(1_700_000_000), (2023, 11, 14, 22, 13, 20));
        assert_eq!(utc_parts(-1), (1969, 12, 31, 23, 59, 59));
    }

    #[test]
    fn relative_time_buckets() {
        let now = 1_700_000_000;
        assert_eq!(relative_time(now, now), "just now");
        assert_eq!(relative_time(now + 500, now), "just now");
        assert_eq!(relative_time(now - 125, now), "2m ago");
        assert_eq!(relative_time(now - 7_200, now), "2h ago");
        assert_eq!(relative_time(now - 3 * 86_400, now), "3d ago");
        assert_eq!(relative_time(951_782_400, now), "Feb 29, 2000");
    }

    #[test]
    fn excerpt_respects_char_boundaries() {
        assert_eq!(excerpt("short", 10), "short");
        assert_eq!(excerpt("héllo wörld", 6), "héllo…");
        assert_eq!(excerpt("🦀🦀🦀", 2), "🦀🦀…");
    }

    #[test]
    fn page_escapes_user_content() {
        let board = models::Board {
            id: 1,
            name: "<script>".to_string(),
            slug: "x".to_string(),
            description: "a & b".to_string(),
            active: true,
        };
        let html = board_page(&board, &[]).into_string();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
