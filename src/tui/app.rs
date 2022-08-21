use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::sync::Arc;

use cursive::event::{Event, Key};
use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::traits::{Nameable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::views::{Dialog, TextView, ViewRef};
use cursive::Cursive;
use cursive::XY;

use super::markdown;
use super::markdown::Markdown;
use super::views::{
    LayoutView, ListView, MdView, Name, TempView, Vimable, NAME_ANSWER_LIST, NAME_ANSWER_VIEW,
    NAME_FULL_LAYOUT, NAME_QUESTION_LIST, NAME_QUESTION_VIEW, NAME_TEMP_MSG,
};
use crate::config::Config;
use crate::error::Result;
use crate::stackexchange::{Answer, Question};

pub const NAME_HELP_VIEW: &str = "help_view";

// TODO an Arc<Mutex> app state that gets auto updated with new selections would
// be convenient

pub fn run(qs: Vec<Question<Markdown>>, cfg: Config) -> Result<()> {
    let mut siv = cursive::default();
    siv.load_theme_file(Config::theme_file_path()?).unwrap(); // TODO dont unwrap

    let question_map: HashMap<u32, Question<Markdown>> =
        qs.clone().into_iter().map(|q| (q.id, q)).collect();
    let question_map = Arc::new(question_map);
    let answer_map: HashMap<u32, Answer<Markdown>> = qs
        .clone()
        .into_iter()
        .flat_map(|q| q.answers.into_iter().map(|a| (a.id, a)))
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

    // Help / View keymappings
    siv.add_global_callback('?', |s| {
        if let Some(pos) = s.screen_mut().find_layer_from_name(NAME_HELP_VIEW) {
            s.screen_mut().remove_layer(pos);
        } else {
            s.add_layer(help());
        }
    });

    // Reload theme
    siv.add_global_callback(Event::CtrlChar('r'), |s| {
        s.load_theme_file(Config::theme_file_path().unwrap())
            .unwrap()
    });

    // Copy contents to sys clipboard
    siv.add_global_callback('y', move |s| {
        let mut v: ViewRef<LayoutView> = s
            .find_name(NAME_FULL_LAYOUT)
            .expect("bug: layout view should exist");
        let md = v.get_focused_content();
        if let Some(mut copy_cmd) = cfg.get_copy_cmd() {
            let res = (|| {
                let mut child = copy_cmd.spawn().map_err(|e| {
                    if e.kind() == io::ErrorKind::NotFound {
                        io::Error::new(
                            io::ErrorKind::Other,
                            "couldn't exec copy cmd; you may need to configure it manually",
                        )
                    } else {
                        e
                    }
                })?;
                let mut stdin = child.stdin.take().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::Other, "couldn't get stdin of copy cmd")
                })?;
                stdin.write_all(md.source().as_bytes())?;
                Ok("copied to clipboard!".to_string())
            })();
            temp_feedback_msg(s, res);
        }
    });

    siv.add_global_callback(Event::Key(Key::Esc), |s| {
        if let Some(pos) = s.screen_mut().find_layer_from_name(NAME_HELP_VIEW) {
            s.screen_mut().remove_layer(pos);
        }
        if let Some(pos) = s.screen_mut().find_layer_from_name(NAME_TEMP_MSG) {
            s.screen_mut().remove_layer(pos);
        }
    });

    // Run the app
    siv.run();
    Ok(())
}

fn question_selected_callback(
    question_map: Arc<HashMap<u32, Question<Markdown>>>,
    s: &mut Cursive,
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
    cb(s)
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

// This would be a good usecase for brining in termimad tables
pub fn help() -> Dialog {
    let bindings = r###"
## Panes
**Tab**:   Focus next pane
**Space**: Cycle layout (4 Pane, 2 Pane, FullScreen)

## Scroll
**h,j,k,l**: ←,↓,↑,→
**Ctrl<u>**: ↑ x 5
**Ctrl<d>**: ↓ x 5
**Ctrl<b>**: ↑ x 10
**Ctrl<f>**: ↓ x 10
**gg**:      Scroll To Top
**G**:       Scroll To Bottom

## Misc
**y**:              Copy current q/a to the clipboard
**q, ZZ, Ctrl<c>**: Exit
**Ctrl<r>**:        Reload theme
**?**:              Toggle this help menu
"###;
    Dialog::around(
        TextView::new(markdown::parse(bindings))
            .scrollable()
            .with_name(NAME_HELP_VIEW),
    )
    .dismiss_button("Close")
    .title("Help")
}

pub fn temp_feedback_msg(siv: &mut Cursive, msg: io::Result<String>) {
    // TODO semaphore to close existing msg before displaying new one
    let style = if msg.is_ok() {
        Color::Light(BaseColor::Green)
    } else {
        Color::Light(BaseColor::Red)
    };
    let content = msg.unwrap_or_else(|e| format!("error: {}", e));
    let styled_content = SpannedString::styled(content, style);
    let layer = Dialog::around(TextView::new(styled_content));
    let temp = TempView::new(layer, siv.cb_sink().clone());
    siv.add_layer(temp);
}

// TODO see cursive/examples/src/bin/select_test.rs for how to test the interface!
// maybe see if we can conditionally run when --nocapture is passed?
