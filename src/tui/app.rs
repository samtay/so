use cursive::event::EventResult;
use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::traits::Nameable;
use cursive::utils::span::SpannedString;
use cursive::views::{
    LinearLayout, NamedView, OnEventView, Panel, ResizedView, SelectView, TextContent, TextView,
};
use cursive::XY;
use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;

use super::markdown;
use crate::config;
use crate::error::Result;
use crate::stackexchange::{Answer, Question};

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

// TODO make my own views for lists, md, etc, and use cursive::inner_getters!
// (or at least type synonyms)
// and abstract out the common builder funcs

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
    siv.load_theme_file(config::theme_file_name()?).unwrap(); // TODO dont unwrap
    let XY { x, y } = siv.screen_size();

    //app state
    //put this in siv.set_user_data? hmm
    //TODO maybe this isn't necessary until multithreading

    let question_map: HashMap<u32, Question> = qs.clone().into_iter().map(|q| (q.id, q)).collect();
    let question_map = Arc::new(question_map);
    let answer_map: HashMap<u32, Answer> = qs
        .clone()
        .into_iter()
        .map(|q| q.answers.into_iter().map(|a| (a.id, a)))
        .flatten()
        .collect();
    let answer_map = Arc::new(answer_map);

    // question view
    let current_question = TextContent::new(""); // init would be great
    let question_view: NamedView<TextView> =
        TextView::new_with_content(current_question.clone()).with_name("question");
    let question_view = Panel::new(question_view);

    // answer view
    let current_answer = TextContent::new(""); // init would be great
    let answer_view: NamedView<TextView> =
        TextView::new_with_content(current_answer.clone()).with_name("answer");
    let answer_view = Panel::new(answer_view);

    // question list view
    //let question_map_ = question_map.clone();
    //let current_question_ = current_question.clone();
    // TODO fuck select view has indexing capabilities :facepalm:
    let mut question_list_view: NamedView<SelectView<u32>> = SelectView::new()
        .with_all(qs.into_iter().map(|q| (q.title, q.id)))
        .on_select(move |mut s, qid| {
            let q = question_map.get(qid).unwrap();
            let XY { x, y: _y } = s.screen_size();
            current_question.set_content(markdown::parse(&q.body));
            let cb = s.call_on_name("answer_list", move |v: &mut SelectView<u32>| {
                v.clear();
                v.add_all(q.answers.iter().map(|a| {
                    // TODO make a damn func for this
                    // add score & accepted checkmark
                    let width = cmp::min(a.body.len(), x / 2);
                    let a_body = a.body[..width].to_owned();
                    let md = markdown::preview(x, a_body);
                    let color = if a.score > 0 {
                        Color::Light(BaseColor::Green)
                    } else {
                        Color::Light(BaseColor::Red)
                    };
                    let mut preview = SpannedString::styled(
                        format!("({}) ", a.score),
                        Style::merge(&[Style::from(color), Style::from(Effect::Bold)]),
                    );
                    if a.is_accepted {
                        preview.append_styled(
                            "\u{2713} ", // "✔ "
                            Style::merge(&[
                                Style::from(Color::Light(BaseColor::Green)),
                                Style::from(Effect::Bold),
                            ]),
                        );
                    }
                    preview.append(md);
                    (preview, a.id)
                }));
                v.set_selection(0)
            });
            if let Some(cb) = cb {
                cb(&mut s)
            }
        })
        .with_name("question_list");
    let question_list_view = make_select_scrollable(question_list_view);
    let question_list_view = Panel::new(question_list_view).title("Questions");

    // answer list view
    //let answer_map_ = answer_map.clone();
    //let current_answer_ = current_question.clone();
    let answer_list_view: NamedView<SelectView<u32>> = SelectView::new()
        .on_select(move |_, aid| {
            let a = answer_map.get(aid).unwrap();
            current_answer.set_content(markdown::parse(&a.body));
        })
        .with_name("answer_list");
    let answer_list_view = make_select_scrollable(answer_list_view);
    let answer_list_view = Panel::new(answer_list_view).title("Answers");

    //TODO eventually do this in the right place, e.g. abstract out md
    //parser, write benches, & do within threads
    siv.add_layer(
        LinearLayout::horizontal()
            .child(ResizedView::with_fixed_width(
                x / 2,
                LinearLayout::vertical()
                    .child(ResizedView::with_fixed_height(y / 3, question_list_view))
                    .child(ResizedView::with_fixed_height(2 * y / 3, question_view)),
            ))
            .child(ResizedView::with_fixed_width(
                x / 2,
                LinearLayout::vertical()
                    .child(ResizedView::with_fixed_height(y / 3, answer_list_view))
                    .child(ResizedView::with_fixed_height(2 * y / 3, answer_view)),
            )),
    );
    let cb = siv.call_on_name("question_list", |v: &mut SelectView<u32>| {
        v.set_selection(0)
    });
    if let Some(cb) = cb {
        cb(&mut siv)
    }
    cursive::logger::init();
    siv.add_global_callback('?', cursive::Cursive::toggle_debug_console);
    siv.run();
    Ok(())
}

// TODO move this out to utils
// use LastSizeView if i want to resize things with shift <HJKL>
// Also, it might be that we control all scrolling from the top
fn make_select_scrollable(
    view: NamedView<SelectView<u32>>,
) -> OnEventView<NamedView<SelectView<u32>>> {
    // Clobber existing functionality:
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            Some(EventResult::Consumed(Some(s.get_mut().select_up(1))))
        })
        .on_pre_event_inner('j', |s, _| {
            Some(EventResult::Consumed(Some(s.get_mut().select_down(1))))
        })
}

// TODO see cursive/examples/src/bin/select_test.rs for how to test the interface!
// maybe see if we can conditionally run when --nocapture is passed?
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
    }
}
