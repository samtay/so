use cursive::views::TextView;

use super::markdown;
use crate::error::Result;
use crate::stackexchange::Question;

// -----------------------------------------
// |question title list|answer preview list| 1/3
// -----------------------------------------
// |question body      |answer body        | 2/3
// -----------------------------------------
// TODO <shift+HJKL> moves layout boundaries
// TODO <hjkl> to move focus? at least for lists..
// TODO <space> to cycle layout
// TODO <?> to bring up key mappings
// TODO query initial term size to choose initial layout

// TODO Circular Focus handles layout & focus & stuff
// TODO these might be "layers" ?
pub enum Layout {
    BothColumns,
    SingleColumn,
    FullScreen,
}

// Tab to cycle focus
// TODO use NamedView
pub enum Focus {
    QuestionList,
    AnswerList,
    Question,
    Answer,
}

pub enum Mode {
    /// Akin to vim, keys are treated as commands
    Normal,
    /// Akin to vim, user is typing in bottom prompt
    Insert,
    // TODO if adding a search feature, that will be anther mode
}

//pub struct App<'a> {
//pub stackexchange: StackExchange,
///// the questions matching the current query
//pub question_list: StatefulList<Question>,
///// the answers to a single question (i.e. the answer list currently shown)
//pub answer_list: StatefulList<Answer>,
//pub questions: Vec<Question>,
//pub layout: Layout,
//pub focus: Focus,
//pub mode: Mode,
//pub ratio: (u32, u32),
//}

// TODO maybe a struct like Tui::new(stackexchange) creates App::new and impls tui.run()?
// TODO views::SelectView?
// TODO take async questions
// TODO take the entire SE struct for future questions
pub fn run(qs: Vec<Question>) -> Result<()> {
    let mut siv = cursive::default();

    //TODO eventually do this in the right place, e.g. abstract out md
    //parser, write benches, & do within threads
    let md = markdown::parse(
        qs[0].answers[0]
            .body
            .clone()
            .replace("<kbd>", "**[")
            .replace("</kbd>", "]**"),
    );
    siv.add_layer(TextView::new(md));

    siv.run();
    Ok(())
}

// TODO prettier and more valuable tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stackexchange::{Answer, Question};
    #[test]
    fn test_app() {
        let ans_body = r#"
Also try the iter:
1. asdf
2. asfd
0. asdfa sfsdf

but

    cargo build --example stderr

and then you run it with

    cd "$(target/debug/examples/stderr)"
    cd `(target/debug/examples/stderr)`

what the application prints on stdout is used as argument to `cd`.

Try it out.

Hit any key to quit this screen:

* **1** will print `..`
* **2** will print `/`
* **3** will print `~`
* or anything else to print this text (so that you may copy-paste)
"#;
        let qs = vec![Question {
            id: 42,
            score: 323,
            title: "How do I exit Vim?".to_string(),
            body: "yo this be my problem dawg but don't say **do** `this`".to_string(),
            answers: vec![
                Answer {
                    id: 422,
                    score: -4,
                    body: ans_body.to_string(),
                    is_accepted: false,
                },
                Answer {
                    id: 423,
                    score: 23,
                    body: "this is a *good* answer tho".to_string(),
                    is_accepted: true,
                },
            ],
        }];

        assert_eq!(run(qs).unwrap(), ());
    }
}
