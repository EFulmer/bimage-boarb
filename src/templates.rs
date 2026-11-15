use std::time::{SystemTime, UNIX_EPOCH};

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::models;

// Inlined into every page so serving it doesn't need its own route.
const STYLESHEET: &str = include_str!("../static/style.css");
// Inline so browsers don't request /favicon.ico. `#` must be escaped as %23 inside a data URI.
const FAVICON: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'>\
<rect width='32' height='32' rx='7' fill='%23b5400c'/>\
<text x='14' y='24' text-anchor='middle' font-family='Georgia,serif' font-weight='700' font-size='22' fill='%23fffcf5'>b</text>\
<circle cx='25' cy='21' r='2.5' fill='%23fffcf5'/></svg>";
const EXCERPT_CHARS: usize = 280;

pub fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · bimage-boarb" }
                link rel="icon" type="image/svg+xml" href=(FAVICON);
                style { (PreEscaped(STYLESHEET)) }
            }
            body {
                a.skip-link href="#main" { "Skip to content" }
                header.site-header {
                    div.wrap {
                        a.wordmark href="/" { "bimage" span.dot { "·" } "boarb" }
                        nav aria-label="Site" {
                            a href="/" { "Boards" }
                        }
                    }
                }
                main #main .wrap { (body) }
                footer.site-footer {
                    div.wrap { "Have a nice day! ☺️" }
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
                            a.board-card href={ "/" (board.slug) } {
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

pub fn board_page(
    board: &models::Board,
    threads: &[models::ThreadSummary],
    form: &ThreadForm,
) -> Markup {
    layout(
        &board.name,
        html! {
            (board_header(board))
            (new_thread_form(board, form))
            @if threads.is_empty() {
                p.empty { "No threads yet." }
            } @else {
                ol.thread-list {
                    @for thread in threads {
                        li {
                            article.thread-summary {
                                h2 {
                                    a href={ "/" (board.slug) "/threads/" (thread.id) } { (thread.title) }
                                }
                                p.meta {
                                    "Started " (timestamp(thread.created_at))
                                    " · last bump " (timestamp(thread.bumped_at))
                                }
                                @if !thread.first_post_body.trim().is_empty() {
                                    p.excerpt { (excerpt(&thread.first_post_body, EXCERPT_CHARS)) }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

/// A validation problem with one form field, shown next to that field and in the error summary.
/// `field` is the form field's `name` attribute, e.g. `"body"`.
#[derive(Debug, Default)]
pub struct FieldError {
    pub field: &'static str,
    pub message: String,
}

/// What the reply form shows. Use `ReplyForm::default()` for a fresh, empty form; after a
/// failed POST, pass back what the user typed plus the errors, so nothing they wrote is lost.
#[derive(Debug, Default)]
pub struct ReplyForm<'a> {
    pub screen_name: &'a str,
    pub body: &'a str,
    pub errors: &'a [FieldError],
}

/// What the new-thread form on the board page shows. Same idea as `ReplyForm`, plus a title.
/// Error `field`s are `"title"`, `"screen_name"` and `"body"`.
#[derive(Debug, Default)]
pub struct ThreadForm<'a> {
    pub title: &'a str,
    pub screen_name: &'a str,
    pub body: &'a str,
    pub errors: &'a [FieldError],
}

fn error_for<'e>(errors: &'e [FieldError], field: &str) -> Option<&'e str> {
    errors
        .iter()
        .find(|e| e.field == field)
        .map(|e| e.message.as_str())
}

/// `posts` is the whole thread in display order: the first post is the opening post, shown under
/// the title, and the rest are replies.
pub fn thread_page(
    board: &models::Board,
    thread: &models::Thread,
    posts: &[models::Post],
    form: &ReplyForm,
) -> Markup {
    let (opening_post, replies) = match posts.split_first() {
        Some((first, rest)) => (Some(first), rest),
        None => (None, posts),
    };
    layout(
        &thread.title,
        html! {
            nav.breadcrumbs aria-label="Breadcrumb" {
                a href="/" { "Boards" }
                " / "
                a href={ "/" (board.slug) } { "/" (board.slug) "/" }
            }
            article.thread id=[opening_post.map(|op| format!("p{}", op.id))] {
                header.op {
                    h1.page-title { (thread.title) }
                    p.meta {
                        @if let Some(op) = opening_post {
                            span.screen-name { (screen_name(op)) }
                            " · "
                        }
                        "Started " (timestamp(thread.created_at))
                        @if let Some(op) = opening_post {
                            " · "
                            a.post-number href={ "#p" (op.id) } title="Link to this post" { "No. " (op.id) }
                        }
                    }
                }
                @if let Some(op) = opening_post {
                    (post_body(&op.body))
                }
            }
            section.replies aria-label="Replies" {
                h2.replies-heading { (reply_count_label(replies.len())) }
                @for post in replies { (post_card(post)) }
            }
            (reply_form(board, thread, form))
        },
    )
}

/// The `#reply` fragment on the action means that when the server re-renders the page with
/// errors, the browser lands on the form instead of the top of a long thread.
fn reply_form(board: &models::Board, thread: &models::Thread, form: &ReplyForm) -> Markup {
    html! {
        section.reply-section #reply aria-labelledby="reply-heading" {
            h2.replies-heading #reply-heading { "Reply" }
            (error_summary("reply", "Your reply wasn't posted:", form.errors))
            form.post-form method="post" action={ "/" (board.slug) "/threads/" (thread.id) "#reply" } {
                (name_field("reply", form.screen_name, error_for(form.errors, "screen_name")))
                (body_field("reply", form.body, error_for(form.errors, "body")))
                button.button type="submit" { "Post reply" }
            }
        }
    }
}

/// Collapsed behind a "Start a new thread" toggle so it doesn't push the thread list down; it
/// opens by itself when the server re-renders it with errors.
fn new_thread_form(board: &models::Board, form: &ThreadForm) -> Markup {
    let title_error = error_for(form.errors, "title");
    html! {
        details.new-thread #new-thread open[!form.errors.is_empty()] {
            summary { "Start a new thread" }
            (error_summary("thread", "Your thread wasn't posted:", form.errors))
            form.post-form method="post" action={ "/" (board.slug) "#new-thread" } {
                div.field {
                    label for="thread-title" { "Title" }
                    input #thread-title type="text" name="title" required
                        value=(form.title)
                        maxlength=(models::MAX_TITLE_CHARS)
                        aria-invalid=[title_error.map(|_| "true")]
                        aria-describedby=[title_error.map(|_| "thread-title-error")];
                    @if let Some(message) = title_error {
                        p.field-error #thread-title-error { (message) }
                    }
                }
                (name_field("thread", form.screen_name, error_for(form.errors, "screen_name")))
                (body_field("thread", form.body, error_for(form.errors, "body")))
                button.button type="submit" { "Start thread" }
            }
        }
    }
}

/// Lists every problem as a link to its field. `prefix` is the form's id prefix ("reply",
/// "thread"), so the links point at `#{prefix}-{field}`.
fn error_summary(prefix: &str, title: &str, errors: &[FieldError]) -> Markup {
    html! {
        @if !errors.is_empty() {
            div.error-summary role="alert" {
                p.error-summary-title { (title) }
                ul {
                    @for error in errors {
                        li { a href={ "#" (prefix) "-" (error.field) } { (error.message) } }
                    }
                }
            }
        }
    }
}

fn name_field(prefix: &str, value: &str, error: Option<&str>) -> Markup {
    let id = format!("{prefix}-screen_name");
    let error_id = format!("{id}-error");
    html! {
        div.field {
            label for=(id) {
                "Name " span.optional { "(optional)" }
            }
            input #(id) type="text" name="screen_name"
                value=(value)
                maxlength=(models::MAX_NAME_CHARS)
                placeholder=(models::DEFAULT_SCREEN_NAME)
                autocomplete="nickname"
                aria-invalid=[error.map(|_| "true")]
                aria-describedby=[error.map(|_| &error_id)];
            @if let Some(message) = error {
                p.field-error #(error_id) { (message) }
            }
        }
    }
}

fn body_field(prefix: &str, value: &str, error: Option<&str>) -> Markup {
    let id = format!("{prefix}-body");
    html! {
        div.field {
            label for=(id) { "Comment" }
            textarea #(id) name="body" rows="6" required
                maxlength=(models::MAX_BODY_CHARS)
                aria-invalid=[error.map(|_| "true")]
                aria-describedby={
                    @if error.is_some() { (id) "-error " }
                    (id) "-hint"
                } { (value) }
            @if let Some(message) = error {
                p.field-error id={ (id) "-error" } { (message) }
            }
            p.hint id={ (id) "-hint" } {
                "Start a line with " code { ">" } " to quote, and use "
                code { ">>123" } " to link to post No. 123."
            }
        }
    }
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
                p { a href="/" { "Back to the boards" } }
            }
        },
    )
}

fn board_header(board: &models::Board) -> Markup {
    html! {
        header.board-header {
            p.slug { a href={ "/" (board.slug) } { "/" (board.slug) "/" } }
            h1.page-title { (board.name) }
            p.board-description { (board.description) }
        }
    }
}

/// The post's screen name, or the default for anonymous posts.
fn screen_name(post: &models::Post) -> &str {
    post.screen_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(models::DEFAULT_SCREEN_NAME)
}

fn post_card(post: &models::Post) -> Markup {
    html! {
        article.post id={ "p" (post.id) } {
            header.post-header {
                span.screen-name { (screen_name(post)) }
                (timestamp(post.created_at))
                a.post-number href={ "#p" (post.id) } title="Link to this post" { "No. " (post.id) }
            }
            @if let Some(parent) = post.in_reply_to {
                p.reply-to { a href={ "#p" (parent) } { ">>" (parent) } }
            }
            (post_body(&post.body))
        }
    }
}

/// Renders a post body with imageboard-style markup. Lines keep their breaks (`.post-body` is
/// `pre-wrap`); a line starting with `>` is a quote, `>>123` links to post 123, and http(s)
/// URLs become links. Everything else goes through maud's escaping like any other text.
fn post_body(body: &str) -> Markup {
    html! {
        div.post-body {
            @for (i, line) in body.lines().enumerate() {
                @if i > 0 { "\n" }
                @if is_quote_line(line) {
                    span.quote { (inline(line)) }
                } @else {
                    (inline(line))
                }
            }
        }
    }
}

/// `>text` is a quote, but a line that opens with a `>>123` reference is not.
fn is_quote_line(line: &str) -> bool {
    line.starts_with('>') && post_ref(line).is_none()
}

/// A piece of one line of a post body.
#[derive(Debug, PartialEq)]
enum Span<'a> {
    Text(&'a str),
    PostRef(i64),
    Url(&'a str),
}

fn inline(line: &str) -> Markup {
    html! {
        @for span in spans(line) {
            @match span {
                Span::Text(text) => (text),
                Span::PostRef(id) => a.post-ref href={ "#p" (id) } { ">>" (id) },
                Span::Url(url) => a href=(url) rel="nofollow noopener noreferrer ugc" { (url) },
            }
        }
    }
}

/// Splits a line into plain text, `>>123` references and URLs, scanning left to right.
fn spans(line: &str) -> Vec<Span<'_>> {
    let mut out = Vec::new();
    let mut text_start = 0;
    let mut i = 0;
    while i < line.len() {
        let rest = &line[i..];
        let after_word = line[..i]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let found = post_ref(rest).or_else(|| if after_word { None } else { url(rest) });
        match found {
            Some((len, span)) => {
                if text_start < i {
                    out.push(Span::Text(&line[text_start..i]));
                }
                out.push(span);
                i += len;
                text_start = i;
            }
            // Step a whole character so `i` always stays on a UTF-8 boundary.
            None => i += rest.chars().next().map_or(1, char::len_utf8),
        }
    }
    if text_start < line.len() {
        out.push(Span::Text(&line[text_start..]));
    }
    out
}

/// `>>` followed by digits, returning the matched length and the post id.
fn post_ref(s: &str) -> Option<(usize, Span<'_>)> {
    let digits = s.strip_prefix(">>")?;
    let len = digits.bytes().take_while(u8::is_ascii_digit).count();
    let id = digits[..len].parse().ok()?;
    Some((2 + len, Span::PostRef(id)))
}

/// An http(s) URL running to the next whitespace, minus trailing punctuation that's more
/// likely to end the sentence than the URL. A `)` is kept when it closes a `(` inside the
/// URL, as in Wikipedia links.
fn url(s: &str) -> Option<(usize, Span<'_>)> {
    let scheme = ["https://", "http://"]
        .into_iter()
        .find(|p| s.starts_with(p))?;
    let mut end = s.find(char::is_whitespace).unwrap_or(s.len());
    loop {
        let candidate = &s[..end];
        match candidate.chars().next_back() {
            Some('.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"') => end -= 1,
            Some(')') if candidate.matches('(').count() < candidate.matches(')').count() => {
                end -= 1
            }
            _ => break,
        }
    }
    (end > scheme.len()).then(|| (end, Span::Url(&s[..end])))
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
    fn spans_find_refs_and_urls() {
        assert_eq!(spans("plain"), vec![Span::Text("plain")]);
        assert_eq!(
            spans("see >>12, ok"),
            vec![Span::Text("see "), Span::PostRef(12), Span::Text(", ok")]
        );
        assert_eq!(spans(">>"), vec![Span::Text(">>")]);
        assert_eq!(spans(">>>5"), vec![Span::Text(">"), Span::PostRef(5)]);
        assert_eq!(
            spans("go to https://example.com/a?b=1."),
            vec![
                Span::Text("go to "),
                Span::Url("https://example.com/a?b=1"),
                Span::Text(".")
            ]
        );
        assert_eq!(
            spans("(https://en.wikipedia.org/wiki/Rust_(language))"),
            vec![
                Span::Text("("),
                Span::Url("https://en.wikipedia.org/wiki/Rust_(language)"),
                Span::Text(")"),
            ]
        );
        assert_eq!(spans("https://"), vec![Span::Text("https://")]);
        assert_eq!(spans("xhttp://a.b"), vec![Span::Text("xhttp://a.b")]);
        assert_eq!(
            spans("🦀 http://a.b"),
            vec![Span::Text("🦀 "), Span::Url("http://a.b")]
        );
    }

    #[test]
    fn post_body_quotes_and_escapes() {
        let html = post_body(">be me\n>>3 hi\n<b>").into_string();
        assert_eq!(
            html,
            "<div class=\"post-body\"><span class=\"quote\">&gt;be me</span>\n\
             <a class=\"post-ref\" href=\"#p3\">&gt;&gt;3</a> hi\n&lt;b&gt;</div>"
        );
        // Only http(s) is linked, so there's no way to smuggle in a javascript: href.
        let html = post_body("javascript:alert(1) https://x.y/\"onmouseover=").into_string();
        assert!(!html.contains("href=\"javascript"));
        assert!(html.contains("href=\"https://x.y/&quot;onmouseover=\""));
    }

    fn sample_board_and_thread() -> (models::Board, models::Thread) {
        let board = models::Board {
            id: 1,
            name: "Main".to_string(),
            slug: "main".to_string(),
            description: "d".to_string(),
            active: true,
        };
        let thread = models::Thread {
            id: 7,
            board_id: 1,
            created_at: 0,
            bumped_at: 0,
            active: true,
            title: "t".to_string(),
        };
        (board, thread)
    }

    fn sample_post(id: i64, body: &str) -> models::Post {
        models::Post {
            id,
            thread_id: 7,
            created_at: 0,
            active: true,
            image_id: None,
            screen_name: None,
            body: body.to_string(),
            in_reply_to: None,
        }
    }

    #[test]
    fn thread_page_splits_opening_post_from_replies() {
        let (board, thread) = sample_board_and_thread();
        let posts = [sample_post(1, "opening"), sample_post(2, "a reply")];
        let html = thread_page(&board, &thread, &posts, &ReplyForm::default()).into_string();
        // The opening post is the thread article itself, so `>>1` and `#p1` links land on it.
        assert!(html.contains("<article class=\"thread\" id=\"p1\">"));
        assert!(html.contains("opening"));
        // Only real replies are counted and rendered as reply cards.
        assert!(html.contains("1 reply"));
        assert_eq!(html.matches("class=\"post\"").count(), 1);
        assert!(html.contains("No. 1"));
    }

    #[test]
    fn thread_page_without_posts_still_renders() {
        let (board, thread) = sample_board_and_thread();
        let html = thread_page(&board, &thread, &[], &ReplyForm::default()).into_string();
        assert!(html.contains("<article class=\"thread\">"));
        assert!(html.contains("No replies yet"));
    }

    #[test]
    fn reply_form_fresh() {
        let (board, thread) = sample_board_and_thread();
        let html = reply_form(&board, &thread, &ReplyForm::default()).into_string();
        assert!(html.contains("action=\"/main/threads/7#reply\""));
        assert!(html.contains("maxlength=\"10000\""));
        assert!(html.contains("maxlength=\"64\""));
        assert!(!html.contains("error-summary"));
        assert!(!html.contains("aria-invalid"));
        assert!(html.contains("aria-describedby=\"reply-body-hint\""));
    }

    #[test]
    fn reply_form_keeps_input_and_shows_errors() {
        let (board, thread) = sample_board_and_thread();
        let errors = [FieldError {
            field: "body",
            message: "Write something first.".to_string(),
        }];
        let form = ReplyForm {
            screen_name: "<anon>",
            body: "",
            errors: &errors,
        };
        let html = reply_form(&board, &thread, &form).into_string();
        assert!(html.contains("role=\"alert\""));
        assert!(html.contains("href=\"#reply-body\""));
        assert!(html.contains("value=\"&lt;anon&gt;\""));
        assert!(html.contains("aria-describedby=\"reply-body-error reply-body-hint\""));
        // Only the field with an error is marked invalid.
        assert_eq!(html.matches("aria-invalid=\"true\"").count(), 1);
    }

    #[test]
    fn new_thread_form_fresh_is_collapsed() {
        let (board, _) = sample_board_and_thread();
        let html = new_thread_form(&board, &ThreadForm::default()).into_string();
        assert!(html.contains("<details class=\"new-thread\" id=\"new-thread\">"));
        assert!(html.contains("action=\"/main#new-thread\""));
        assert!(html.contains("name=\"title\""));
        assert!(html.contains("maxlength=\"150\""));
        assert!(html.contains("id=\"thread-body\""));
        assert!(!html.contains("error-summary"));
    }

    #[test]
    fn new_thread_form_with_errors_opens_and_keeps_input() {
        let (board, _) = sample_board_and_thread();
        let errors = [FieldError {
            field: "title",
            message: "Give your thread a title.".to_string(),
        }];
        let form = ThreadForm {
            title: "",
            screen_name: "",
            body: "kept <text>",
            errors: &errors,
        };
        let html = new_thread_form(&board, &form).into_string();
        assert!(html.contains("<details class=\"new-thread\" id=\"new-thread\" open>"));
        assert!(html.contains("href=\"#thread-title\""));
        assert!(html.contains("aria-describedby=\"thread-title-error\""));
        assert!(html.contains("kept &lt;text&gt;"));
        assert_eq!(html.matches("aria-invalid=\"true\"").count(), 1);
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
        let html = board_page(&board, &[], &ThreadForm::default()).into_string();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
}
