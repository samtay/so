//! Parse markdown text.
//!
//! Extended from cursive::utils::markup::markdown to add code styles
//! TODO: Bring in the full power of termimad (e.g. md tables) in a View;
//! implementation of those features (.e.g automatic wrapping within each table
//! cell) might be easier in this setting anyway.
// Figure out why the hell cursive needs to keep around the input string?

// TODO use ColorStyle::secondary() etc. over specific enums

use cursive::theme::{Effect, PaletteColor, Style};
use cursive::utils::markup::{StyledIndexedSpan, StyledString};
use cursive::utils::span::{IndexedCow, IndexedSpan};
use pulldown_cmark::{self, CowStr, Event, Options, Tag};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

use super::entities::is_entity;

/// Parses the given string as markdown text.
pub fn parse<S>(input: S) -> StyledString
where
    S: Into<String>,
{
    let input = preprocess(input.into());
    let spans = parse_spans(&input);
    //let output = build_output(&spans);
    StyledString::with_spans(input, spans)
}

/// Preview markdown. Largely heuristic.
pub fn preview<S>(size: usize, input: S) -> StyledString
where
    S: Into<String>,
{
    // DO the initial parsing here too, not just in `parse`
    let generous_size = (size as f32) * 1.2;
    let generous_size = generous_size.ceil();
    let generous_size = generous_size as usize;
    let mut input = input.into();
    input.truncate(generous_size);
    let input = preprocess(input);
    let spans = parse_spans(&input)
        .into_iter()
        // Filter out newlines
        .map(|ix_span| match ix_span {
            IndexedSpan { width: 0, .. } => IndexedSpan {
                content: IndexedCow::Owned(" ".to_owned()),
                width: 1,
                ..ix_span
            },
            is => is,
        })
        .collect();

    let mut prev = StyledString::with_spans(input, spans);
    prev.append_plain("...");
    prev
}

fn preprocess(input: String) -> String {
    // TODO handle other stackexchange oddities here ENTITIES
    // TODO then benchmark
    input
        .as_str()
        .trim()
        .replace("<kbd>", "**[")
        .replace("</kbd>", "]**")
}

/// Parse the given markdown text into a list of spans.
/// Assumes preprocessing has taken place
/// This is a shortcut for `Parser::new(preprocessed_input).collect()`.
fn parse_spans(input: &str) -> Vec<StyledIndexedSpan> {
    Parser::new(input).collect()
}

/// Cheat my way through cursive's obscure crap
//fn build_output(spans: Vec<StyledIndexedSpan>) -> String {
//spans.iter().fold("", |mut o, s| {
//o += s.content.to_string();
//})
//}

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
        let mut opts = pulldown_cmark::Options::empty();
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);
        Parser {
            input,
            item: None,
            in_list: false,
            after_code_block: false,
            first: true,
            parser: pulldown_cmark::Parser::new_ext(input, opts),
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

    // Big hack here because cursive nonsense;
    // Do some benchmarking and see if the performance is worse
    // by searching the entity set; if so, just own everything
    fn cowstr_to_span(&self, text: CowStr, style: Option<Style>) -> StyledIndexedSpan {
        let text = match text {
            CowStr::Boxed(text) => Cow::Owned(text.into()),
            CowStr::Inlined(text) => Cow::Owned(text.to_string()),
            // If markdown parsed an HTML entity, own the string to avoid panicking in
            // cursive::utils::span::from_cow
            CowStr::Borrowed(text) if is_entity(text) => Cow::Owned(text.to_string()),
            // Otherwise, borrow
            CowStr::Borrowed(text) => Cow::Borrowed(text),
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

// TODO: `how to reverse a list in Python` broken:
// due to newline problem in pulldowm_cmark and stackexchange differences
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
        let parsed = parse(input);
        let spans: Vec<_> = parsed.spans().into_iter().collect();
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
        let parsed = parse(input);
        let spans: Vec<_> = parsed.spans().into_iter().collect();
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
- out of order
and tasks
- [ ] undone, or
- [x] done!
";
        let parsed = parse(input);
        let spans: Vec<_> = parsed.spans().into_iter().collect();
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
    #[test]
    fn test_from_cow_panic() {
        let input = "
I'm on a Mac running OS&nbsp;X&nbsp;v10.6 (Snow&nbsp;Leopard). I have Mercurial 1.1 installed.\r\n\r\nAfter I hit <kbd>Esc</kbd> to exit insert mode I can't figure out how to save and quit. Hitting <kbd>Ctrl</kbd> + <kbd>C</kbd> shows me instructions that say typing \"quit<enter>\" will write and quit, but it doesn't seem to work.\r\n\r\n\r\n\r\n";
        let parsed = parse(input);
        let spans: Vec<_> = parsed.spans().into_iter().collect();
        let expected_spans = &[
            Span {
                content: "I\'m on a Mac running OS",
                attr: &Style::none(),
                width: 23,
            },
            Span {
                content: "\u{a0}",
                attr: &Style::none(),
                width: 1,
            },
            Span {
                content: "X",
                attr: &Style::none(),
                width: 1,
            },
            Span {
                content: "\u{a0}",
                attr: &Style::none(),
                width: 1,
            },
            Span {
                content: "v10.6 (Snow",
                attr: &Style::none(),
                width: 11,
            },
            Span {
                content: "\u{a0}",
                attr: &Style::none(),
                width: 1,
            },
            Span {
                content: "Leopard). I have Mercurial 1.1 installed.",
                attr: &Style::none(),
                width: 41,
            },
            Span {
                content: "\n\n",
                attr: &Style::none(),
                width: 0,
            },
            Span {
                content: "After I hit ",
                attr: &Style::none(),
                width: 12,
            },
            Span {
                content: "[",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: "Esc",
                attr: &Style::from(Effect::Bold),
                width: 3,
            },
            Span {
                content: "]",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: " to exit insert mode I can\'t figure out how to save and quit. Hitting ",
                attr: &Style::none(),
                width: 70,
            },
            Span {
                content: "[",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: "Ctrl",
                attr: &Style::from(Effect::Bold),
                width: 4,
            },
            Span {
                content: "]",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: " + ",
                attr: &Style::none(),
                width: 3,
            },
            Span {
                content: "[",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: "C",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: "]",
                attr: &Style::from(Effect::Bold),
                width: 1,
            },
            Span {
                content: " shows me instructions that say typing \"quit",
                attr: &Style::none(),
                width: 44,
            },
            Span {
                content: "<enter>",
                attr: &Style::none(),
                width: 7,
            },
            Span {
                content: "\" will write and quit, but it doesn\'t seem to work.",
                attr: &Style::none(),
                width: 51,
            },
        ];

        assert_eq!(spans, expected_spans);
        for (span, expected_span) in spans.iter().zip(expected_spans.iter()) {
            assert_eq!(span, expected_span);
        }
    }
}
