//! A layout policy for swc's writer: JSDoc on its own lines.
//!
//! swc's printer writes a leading block comment followed by a space, so a
//! synthetic JSDoc comment would share its line with the code it documents
//! (`/** The id. */ readonly id: string;`), and the lines of a multi-line
//! comment would not be indented. [`DocLines`] wraps the printer's
//! [`WriteJs`] sink and changes two things, both through the sink's own
//! calls: each newline in a comment's text becomes a printer line (so the
//! printer indents it), and the space the printer writes after a closing
//! `*/` becomes a printer line. Every other call is passed through unchanged;
//! the printer still decides every token.

use swc_common::{BytePos, Span};
use swc_ecma_codegen::text_writer::WriteJs;
use swc_ecma_codegen::Result;

/// The token swc's printer writes to close a block comment.
const COMMENT_CLOSE: &str = "*/";

pub(crate) struct DocLines<W> {
    inner: W,
    after_comment: bool,
}

impl<W: WriteJs> DocLines<W> {
    pub(crate) fn new(inner: W) -> Self {
        Self {
            inner,
            after_comment: false,
        }
    }

    fn token(&mut self) -> &mut W {
        self.after_comment = false;
        &mut self.inner
    }
}

impl<W: WriteJs> WriteJs for DocLines<W> {
    fn increase_indent(&mut self) -> Result {
        self.token().increase_indent()
    }

    fn decrease_indent(&mut self) -> Result {
        self.token().decrease_indent()
    }

    fn write_semi(&mut self, span: Option<Span>) -> Result {
        self.token().write_semi(span)
    }

    fn write_space(&mut self) -> Result {
        if self.after_comment {
            self.after_comment = false;
            self.inner.write_line()
        } else {
            self.inner.write_space()
        }
    }

    fn write_keyword(&mut self, span: Option<Span>, s: &'static str) -> Result {
        self.token().write_keyword(span, s)
    }

    fn write_operator(&mut self, span: Option<Span>, s: &str) -> Result {
        self.token().write_operator(span, s)
    }

    fn write_param(&mut self, s: &str) -> Result {
        self.token().write_param(s)
    }

    fn write_property(&mut self, s: &str) -> Result {
        self.token().write_property(s)
    }

    fn write_line(&mut self) -> Result {
        self.token().write_line()
    }

    fn write_lit(&mut self, span: Span, s: &str) -> Result {
        self.token().write_lit(span, s)
    }

    fn write_comment(&mut self, s: &str) -> Result {
        self.after_comment = false;
        for (index, line) in s.split('\n').enumerate() {
            if index > 0 {
                self.inner.write_line()?;
            }
            self.inner.write_comment(line)?;
        }
        self.after_comment = s == COMMENT_CLOSE;
        Ok(())
    }

    fn write_str_lit(&mut self, span: Span, s: &str) -> Result {
        self.token().write_str_lit(span, s)
    }

    fn write_str(&mut self, s: &str) -> Result {
        self.token().write_str(s)
    }

    fn write_symbol(&mut self, span: Span, s: &str) -> Result {
        self.token().write_symbol(span, s)
    }

    fn write_punct(
        &mut self,
        span: Option<Span>,
        s: &'static str,
        commit_pending_semi: bool,
    ) -> Result {
        self.token().write_punct(span, s, commit_pending_semi)
    }

    fn care_about_srcmap(&self) -> bool {
        self.inner.care_about_srcmap()
    }

    fn add_srcmap(&mut self, pos: BytePos) -> Result {
        self.inner.add_srcmap(pos)
    }

    fn commit_pending_semi(&mut self) -> Result {
        self.inner.commit_pending_semi()
    }

    fn can_ignore_invalid_unicodes(&mut self) -> bool {
        self.inner.can_ignore_invalid_unicodes()
    }
}
