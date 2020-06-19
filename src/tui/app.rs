use cursive::event::Event;
use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::Cursive;
use cursive::XY;
use std::collections::HashMap;
use std::sync::Arc;

use super::markdown;
use super::markdown::Markdown;
use super::views::{
    LayoutView, ListView, MdView, Name, Vimable, NAME_ANSWER_LIST, NAME_ANSWER_VIEW,
    NAME_QUESTION_LIST, NAME_QUESTION_VIEW,
};
use crate::config;
use crate::error::Result;
use crate::stackexchange::{Answer, Question};

pub fn run(qs: Vec<Question<Markdown>>) -> Result<()> {
    let mut siv = cursive::default();
    siv.load_theme_file(config::theme_file_name()?).unwrap(); // TODO dont unwrap

    let question_map: HashMap<u32, Question<Markdown>> =
        qs.clone().into_iter().map(|q| (q.id, q)).collect();
    let question_map = Arc::new(question_map);
    let answer_map: HashMap<u32, Answer<Markdown>> = qs
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
        move |s, qid| question_selected_callback(question_map.clone(), s, *qid),
    );

    let answer_list_view = ListView::new(Name::AnswerList, move |s, aid| {
        let a = answer_map.get(aid).unwrap();
        s.call_on_name(NAME_ANSWER_VIEW, |v: &mut MdView| v.set_content(&a.body));
    });

    siv.add_layer(
        LayoutView::new(
            1,
            question_list_view,
            question_view,
            answer_list_view,
            answer_view,
        )
        .add_vim_bindings(),
    );

    let cb = siv.call_on_name(NAME_QUESTION_LIST, |v: &mut ListView| v.select(0));
    if let Some(cb) = cb {
        cb(&mut siv)
    }
    cursive::logger::init();
    siv.add_global_callback('?', cursive::Cursive::toggle_debug_console);
    siv.add_global_callback(Event::CtrlChar('r'), |s| {
        s.load_theme_file(config::theme_file_name().unwrap())
            .unwrap()
    });
    siv.run();
    Ok(())
}

fn question_selected_callback(
    question_map: Arc<HashMap<u32, Question<Markdown>>>,
    mut s: &mut Cursive,
    qid: u32,
) {
    let q = question_map.get(&qid).unwrap();
    let body = &q.body;
    let XY { x, y: _y } = s.screen_size();
    // Update question view
    s.call_on_name(NAME_QUESTION_VIEW, |v: &mut MdView| {
        v.set_content(body);
    })
    .expect("Panic: setting question view content failed");
    // Update answer list view
    let cb = s
        .call_on_name(NAME_ANSWER_LIST, |v: &mut ListView| {
            v.reset_with_all(q.answers.iter().map(|a| (preview_answer(x, a), a.id)))
        })
        .expect("Panic: setting answer list content failed");
    cb(&mut s)
}

fn preview_question(q: &Question<Markdown>) -> StyledString {
    let mut preview = pretty_score(q.score);
    preview.append_plain(&q.title);
    preview
}

fn preview_answer(screen_width: usize, a: &Answer<Markdown>) -> StyledString {
    let md = markdown::preview(screen_width, &a.body);
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
