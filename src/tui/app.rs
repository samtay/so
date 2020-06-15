use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::Cursive;
use cursive::XY;
use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;

use super::markdown;
use super::views::{
    FullLayout, ListView, MdView, Name, NAME_ANSWER_LIST, NAME_ANSWER_VIEW, NAME_QUESTION_LIST,
    NAME_QUESTION_VIEW,
};
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
// TODO take async questions
// TODO take the entire SE struct for future questions
pub fn run(qs: Vec<Question>) -> Result<()> {
    let mut siv = cursive::default();
    siv.load_theme_file(config::theme_file_name()?).unwrap(); // TODO dont unwrap

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

    let question_view = MdView::new(Name::QuestionView);
    let answer_view = MdView::new(Name::AnswerView);

    let question_list_view = ListView::new_with_items(
        Name::QuestionList,
        qs.into_iter().map(|q| (preview_question(&q), q.id)),
        move |s, qid| question_selected_callback(question_map.clone(), s, qid),
    );

    // TODO init with qs[0].answers ?
    let answer_list_view = ListView::new(Name::AnswerList, move |s, aid| {
        let a = answer_map.get(aid).unwrap();
        s.call_on_name(NAME_ANSWER_VIEW, |v: &mut MdView| v.set_content(&a.body));
    });

    siv.add_layer(FullLayout::new(
        1,
        siv.screen_size(),
        question_list_view,
        question_view,
        answer_list_view,
        answer_view,
    ));

    let cb = siv.call_on_name(NAME_QUESTION_LIST, |v: &mut ListView| v.select(0));
    if let Some(cb) = cb {
        cb(&mut siv)
    }
    cursive::logger::init();
    siv.add_global_callback('?', cursive::Cursive::toggle_debug_console);
    println!("{:?}", siv.debug_name(NAME_QUESTION_VIEW));
    siv.run();
    Ok(())
}

// TODO need to get size of question list view, as this will change depending on layout
fn question_selected_callback(
    question_map: Arc<HashMap<u32, Question>>,
    mut s: &mut Cursive,
    qid: &u32,
) {
    let q = question_map.get(qid).unwrap();
    let XY { x, y: _y } = s.screen_size();
    // Update question view
    s.call_on_name(NAME_QUESTION_VIEW, |v: &mut MdView| {
        v.set_content(&q.body);
    })
    .expect("TODO: make sure this is callable: setting question body on view");
    // Update answer list view
    let cb = s
        .call_on_name(NAME_ANSWER_LIST, |v: &mut ListView| {
            v.reset_with_all(q.answers.iter().map(|a| (preview_answer(x, a), a.id)))
        })
        .expect("TODO why would this ever fail");
    cb(&mut s)
}

fn preview_question(q: &Question) -> StyledString {
    let mut preview = pretty_score(q.score);
    preview.append_plain(&q.title);
    preview
}

fn preview_answer(screen_width: usize, a: &Answer) -> StyledString {
    let width = cmp::min(a.body.len(), screen_width / 2);
    let md = markdown::preview(width, a.body.to_owned());
    let mut preview = pretty_score(a.score);
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
    preview
}

fn pretty_score(score: i32) -> StyledString {
    let color = if score > 0 {
        Color::Light(BaseColor::Green)
    } else {
        Color::Light(BaseColor::Red)
    };
    SpannedString::styled(
        format!("({}) ", score),
        Style::merge(&[Style::from(color), Style::from(Effect::Bold)]),
    )
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
