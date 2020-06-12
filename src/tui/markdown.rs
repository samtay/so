//! Parse markdown text.
//!
//! Extended from cursive::utils::markup::markdown to add code styles
//! TODO: Bring in the full power of termimad (e.g. md tables) in a View;
//! implementation of those features (.e.g automatic wrapping within each table
//! cell) might be easier in this setting anyway.

use std::borrow::Cow;

use cursive::theme::{Effect, PaletteColor, Style};
use cursive::utils::markup::{StyledIndexedSpan, StyledString};
use cursive::utils::span::IndexedCow;

use pulldown_cmark::{self, CowStr, Event, Tag};
use unicode_width::UnicodeWidthStr;

/// Parses the given string as markdown text.
pub fn parse<S>(input: S) -> StyledString
where
    S: Into<String>,
{
    // TODO handle other stackexchange oddities here
    // TODO then benchmark
    let input = input
        .into()
        .replace("<kbd>", "**[")
        .replace("</kbd>", "]**");

    let spans = parse_spans(&input);

    StyledString::with_spans(input, spans)
}

/// Iterator that parse a markdown text and outputs styled spans.
pub struct Parser<'a> {
    first: bool,
    item: Option<u64>,
    in_list: bool,
    after_code_block: bool,
    stack: Vec<Style>,
    input: &'a str,
    parser: pulldown_cmark::Parser<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with the given input text.
    pub fn new(input: &'a str) -> Self {
        Parser {
            input,
            item: None,
            in_list: false,
            after_code_block: false,
            first: true,
            parser: pulldown_cmark::Parser::new(input),
            stack: Vec::new(),
        }
    }

    /// Creates a new span with the given value
    fn literal<S>(&self, text: S) -> StyledIndexedSpan
    where
        S: Into<String>,
    {
        StyledIndexedSpan::simple_owned(text.into(), Style::merge(&self.stack))
    }

    fn cow_to_span(&self, text: Cow<str>, style: Option<Style>) -> StyledIndexedSpan {
        let width = text.width();
        StyledIndexedSpan {
            content: IndexedCow::from_cow(text, self.input),
            attr: style.unwrap_or_else(|| Style::merge(&self.stack)),
            width,
        }
    }

    fn cowstr_to_span(&self, text: CowStr, style: Option<Style>) -> StyledIndexedSpan {
        let text = match text {
            CowStr::Boxed(text) => Cow::Owned(text.into()),
            CowStr::Borrowed(text) => Cow::Borrowed(text),
            CowStr::Inlined(text) => Cow::Owned(text.to_string()),
        };
        self.cow_to_span(text, style)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = StyledIndexedSpan;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = match self.parser.next() {
                None => return None,
                Some(event) => event,
            };

            // TODO fix list tag
            match next {
                Event::Start(tag) => match tag {
                    // Add to the stack!
                    Tag::Emphasis => self.stack.push(Style::from(Effect::Italic)),
                    Tag::Heading(level) if level == 1 => {
                        self.stack.push(Style::from(PaletteColor::TitlePrimary))
                    }
                    Tag::Heading(_) => self.stack.push(Style::from(PaletteColor::TitleSecondary)),
                    // TODO style quote?
                    Tag::BlockQuote => return Some(self.literal("> ")),
                    Tag::Link(_, _, _) => return Some(self.literal("[")),
                    Tag::CodeBlock(_) => {
                        self.stack.push(Style::from(PaletteColor::Secondary));
                        return Some(self.literal("\n\n"));
                    }
                    Tag::Strong => self.stack.push(Style::from(Effect::Bold)),
                    Tag::Paragraph if self.after_code_block => {
                        self.after_code_block = false;
                        return Some(self.literal("\n"));
                    }
                    Tag::Paragraph if !self.first && !self.in_list => {
                        return Some(self.literal("\n\n"));
                    }
                    Tag::List(ix) => {
                        self.item = ix;
                        self.in_list = true;
                        if !self.first {
                            return Some(self.literal("\n\n"));
                        }
                    }
                    Tag::Item => match self.item {
                        Some(ix) => {
                            let pre = ix.to_string() + ". ";
                            return Some(
                                self.cow_to_span(Cow::Owned(pre), Some(Style::from(Effect::Bold))),
                            );
                        }
                        None => {
                            return Some(self.literal("• ".to_string()));
                        }
                    },
                    _ => (),
                },
                Event::End(tag) => match tag {
                    // Remove from stack!
                    Tag::Paragraph if self.first => self.first = false,
                    Tag::Heading(_) => {
                        self.stack.pop().unwrap();
                        return Some(self.literal("\n\n"));
                    }
                    // TODO underline the link
                    Tag::Link(_, link, _) => return Some(self.literal(format!("]({})", link))),
                    Tag::CodeBlock(_) => {
                        self.after_code_block = true;
                        self.stack.pop().unwrap();
                    }
                    Tag::Emphasis | Tag::Strong => {
                        self.stack.pop().unwrap();
                    }
                    Tag::List(_) => {
                        self.item = None;
                        self.in_list = false;
                        self.first = false;
                    }
                    Tag::Item => {
                        self.item = self.item.map(|ix| ix + 1);
                        return Some(self.literal("\n"));
                    }
                    _ => (),
                },
                Event::Rule => return Some(self.literal("---")),
                Event::SoftBreak => return Some(self.literal("\n")),
                Event::HardBreak => return Some(self.literal("\n")),
                Event::Code(text) => {
                    return Some(
                        self.cowstr_to_span(text, Some(Style::from(PaletteColor::Secondary))),
                    );
                }
                // Treat all other texts the same
                Event::FootnoteReference(text) | Event::Html(text) | Event::Text(text) => {
                    return Some(self.cowstr_to_span(text, None));
                }
                Event::TaskListMarker(checked) => {
                    let mark = if checked { "[x]" } else { "[ ]" };
                    return Some(self.cow_to_span(
                        Cow::Owned(mark.to_string()),
                        Some(Style::from(Effect::Bold)),
                    ));
                }
            }
        }
    }
}

/// Parse the given markdown text into a list of spans.
///
/// This is a shortcut for `Parser::new(input).collect()`.
pub fn parse_spans(input: &str) -> Vec<StyledIndexedSpan> {
    Parser::new(input).collect()
}

// TODO: how to reverse a list in Python answer is broken; test it here!
#[cfg(test)]
mod tests {
    use super::*;
    use cursive::utils::span::Span;

    #[test]
    fn test_basic_styles() {
        let input = r"
Attention
====
I *really* love __Cursive__!";
        let spans = parse_spans(input);
        let spans: Vec<_> = spans.iter().map(|span| span.resolve(input)).collect();
        let expected_spans = &[
            Span {
                content: "Attention",
                width: 9,
                attr: &Style::from(PaletteColor::TitlePrimary),
            },
            Span {
                content: "\n\n",
                width: 0,
                attr: &Style::none(),
            },
            Span {
                content: "I ",
                width: 2,
                attr: &Style::none(),
            },
            Span {
                content: "really",
                width: 6,
                attr: &Style::from(Effect::Italic),
            },
            Span {
                content: " love ",
                width: 6,
                attr: &Style::none(),
            },
            Span {
                content: "Cursive",
                width: 7,
                attr: &Style::from(Effect::Bold),
            },
            Span {
                content: "!",
                width: 1,
                attr: &Style::none(),
            },
        ];

        for (span, expected_span) in spans.iter().zip(expected_spans.iter()) {
            assert_eq!(span, expected_span);
        }
    }

    #[test]
    fn test_code() {
        let input = r"
## project

Here's some `inline code`. It should escape `*asterisks*`.
It should also respect

    indented code blocks

and
```python
code fences
```
Obviously.";
        let spans = parse_spans(input);
        let spans: Vec<_> = spans.iter().map(|span| span.resolve(input)).collect();
        let expected_spans = &[
            Span {
                content: "project",
                width: 7,
                attr: &Style::from(PaletteColor::TitleSecondary),
            },
            Span {
                content: "\n\n",
                width: 0,
                attr: &Style::none(),
            },
            Span {
                content: "Here's some ",
                width: 12,
                attr: &Style::none(),
            },
            Span {
                content: "inline code",
                width: 11,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: ". It should escape ",
                width: 19,
                attr: &Style::none(),
            },
            Span {
                content: "*asterisks*",
                width: 11,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: ".",
                width: 1,
                attr: &Style::none(),
            },
            Span {
                content: "\n",
                width: 0,
                attr: &Style::none(),
            },
            Span {
                content: "It should also respect",
                width: 22,
                attr: &Style::none(),
            },
            Span {
                content: "\n\n",
                width: 0,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: "indented code blocks\n",
                width: 20,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: "\n",
                width: 0,
                attr: &Style::none(),
            },
            Span {
                content: "and",
                width: 3,
                attr: &Style::none(),
            },
            Span {
                content: "\n\n",
                width: 0,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: "code fences\n",
                width: 11,
                attr: &Style::from(PaletteColor::Secondary),
            },
            Span {
                content: "\n",
                width: 0,
                attr: &Style::none(),
            },
            Span {
                content: "Obviously.",
                width: 10,
                attr: &Style::none(),
            },
        ];

        for (span, expected_span) in spans.iter().zip(expected_spans.iter()) {
            assert_eq!(span, expected_span);
        }
    }

    #[test]
    fn test_lists() {
        let input = r"
1. Do something
0. Then another
or
- do them
- out of order";
        let spans = parse_spans(input);
        let spans: Vec<_> = spans.iter().map(|span| span.resolve(input)).collect();
        let expected_spans = &[
            Span {
                content: "1. ",
                attr: &Style::from(Effect::Bold),
                width: 3,
            },
            Span {
                content: "Do something",
                attr: &Style::none(),
                width: 12,
            },
            Span {
                content: "\n",
                attr: &Style::none(),
                width: 0,
            },
            Span {
                content: "2. ",
                attr: &Style::from(Effect::Bold),
                width: 3,
            },
            Span {
                content: "Then another",
                attr: &Style::none(),
                width: 12,
            },
            Span {
                content: "\n",
                attr: &Style::none(),
                width: 0,
            },
            Span {
                content: "or",
                attr: &Style::none(),
                width: 2,
            },
            Span {
                content: "\n",
                attr: &Style::none(),
                width: 0,
            },
            Span {
                content: "\n\n",
                attr: &Style::none(), // TODO too many newlines
                width: 0,
            },
            Span {
                content: "• ",
                attr: &Style::none(),
                width: 2,
            },
            Span {
                content: "do them",
                attr: &Style::none(),
                width: 7,
            },
            Span {
                content: "\n",
                attr: &Style::none(),
                width: 0,
            },
            Span {
                content: "• ",
                attr: &Style::none(),
                width: 2,
            },
            Span {
                content: "out of order",
                attr: &Style::none(),
                width: 12,
            },
        ];

        for (span, expected_span) in spans.iter().zip(expected_spans.iter()) {
            assert_eq!(span, expected_span);
        }
    }
}
