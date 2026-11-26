//! The text of a JSDoc block comment.
//!
//! swc's printer writes a block comment as `/*`, the comment's text, then
//! `*/`. The text built here is what goes between: a leading `*` that makes
//! the comment JSDoc, and one ` * ` line per line of the doc. Continuation
//! lines carry no indentation; [`crate::writer::DocLines`] breaks them into
//! printer lines so the printer indents each one.

/// The text of the JSDoc block comment that documents a node with `doc`.
///
/// A doc of one line becomes `/** doc */`; a doc of several lines becomes one
/// ` * ` line each between `/**` and ` */`. A `*/` inside the doc would end the
/// comment early, so it is written `*\/`. Line endings are normalised and
/// trailing white space on each line is dropped.
pub(crate) fn comment_text(doc: &str) -> String {
    let safe = doc
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace("*/", "*\\/");
    let lines: Vec<&str> = safe.split('\n').map(str::trim_end).collect();

    let mut text = String::from('*');
    match lines.as_slice() {
        [line] => {
            text.push(' ');
            if !line.is_empty() {
                text.push_str(line);
                text.push(' ');
            }
        }
        _ => {
            for line in &lines {
                text.push_str("\n *");
                if !line.is_empty() {
                    text.push(' ');
                    text.push_str(line);
                }
            }
            text.push_str("\n ");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::comment_text;

    #[test]
    fn one_line_stays_on_one_line() {
        assert_eq!(comment_text("The id."), "* The id. ");
    }

    #[test]
    fn an_empty_doc_is_an_empty_comment() {
        assert_eq!(comment_text(""), "* ");
    }

    #[test]
    fn several_lines_get_a_star_each() {
        assert_eq!(comment_text("a\n\nb  "), "*\n * a\n *\n * b\n ");
    }

    #[test]
    fn carriage_returns_are_line_endings() {
        assert_eq!(comment_text("a\r\nb\rc"), "*\n * a\n * b\n * c\n ");
    }

    #[test]
    fn a_comment_close_in_the_doc_is_escaped() {
        assert_eq!(comment_text("ends */ here"), "* ends *\\/ here ");
    }
}
