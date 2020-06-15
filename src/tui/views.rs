use cursive::event::{Callback, EventResult};
use cursive::traits::{Finder, Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::view::{Margins, SizeConstraint, View, ViewWrapper};
use cursive::views::{
    LinearLayout, NamedView, OnEventView, PaddedView, Panel, ScrollView, SelectView, TextView,
};
use cursive::{Cursive, Vec2};
use std::fmt;
use std::fmt::Display;

use super::markdown;
use crate::error::Result;

pub const NAME_QUESTION_LIST: &str = "question_list";
pub const NAME_ANSWER_LIST: &str = "answer_list";
pub const NAME_QUESTION_VIEW: &str = "question_view";
pub const NAME_ANSWER_VIEW: &str = "answer_view";

// TODO might need resizable wrappers in types

pub enum Name {
    QuestionList,
    AnswerList,
    QuestionView,
    AnswerView,
}

impl Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Name::QuestionList => write!(f, "Questions"),
            Name::AnswerList => write!(f, "Answers"),
            Name::QuestionView => write!(f, "Question"),
            Name::AnswerView => write!(f, "Answer"),
        }
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        match name {
            Name::QuestionList => String::from(NAME_QUESTION_LIST),
            Name::AnswerList => String::from(NAME_ANSWER_LIST),
            Name::QuestionView => String::from(NAME_QUESTION_VIEW),
            Name::AnswerView => String::from(NAME_ANSWER_VIEW),
        }
    }
}

// TODO maybe I should use cursive's ListView over SelectView ?
pub type ListView = ListViewT<Panel<ScrollView<OnEventView<NamedView<SelectView<u32>>>>>>;

pub struct ListViewT<T: View> {
    inner_name: String,
    view: T,
}

impl<T: View> ViewWrapper for ListViewT<T> {
    cursive::wrap_impl!(self.view: T);
}

impl ListView {
    pub fn new<F>(name: Name, on_select: F) -> NamedView<Self>
    where
        F: Fn(&mut Cursive, &u32) + 'static,
    {
        ListView::make_new::<StyledString, Vec<_>, _>(name, None, on_select)
    }

    pub fn new_with_items<S, I, F>(name: Name, items: I, on_select: F) -> NamedView<Self>
    where
        S: Into<StyledString>,
        I: IntoIterator<Item = (S, u32)>,
        F: Fn(&mut Cursive, &u32) + 'static,
    {
        ListView::make_new(name, Some(items), on_select)
    }

    fn make_new<S, I, F>(name: Name, items: Option<I>, on_select: F) -> NamedView<Self>
    where
        S: Into<StyledString>,
        I: IntoIterator<Item = (S, u32)>,
        F: Fn(&mut Cursive, &u32) + 'static,
    {
        let inner_name = name.to_string() + "_inner";
        let mut view = SelectView::new().on_select(on_select);
        if let Some(items) = items {
            view.add_all(items);
        }
        let view = view.with_name(&inner_name);
        let view = add_vim_bindings(view);
        let view = view.scrollable();
        let view = Panel::new(view).title(format!("{}", name));
        let view = ListViewT { view, inner_name };
        view.with_name(name)
    }

    pub fn reset_with_all<S, I>(&mut self, iter: I) -> Callback
    where
        S: Into<StyledString>,
        I: IntoIterator<Item = (S, u32)>,
    {
        self.call_on_inner(|s| {
            s.clear();
            s.add_all(iter);
            s.set_selection(0)
        })
    }

    pub fn select(&mut self, i: usize) -> Callback {
        self.call_on_inner(|sv| sv.set_selection(i))
    }

    fn call_on_inner<F, R>(&mut self, cb: F) -> R
    where
        F: FnOnce(&mut SelectView<u32>) -> R,
    {
        self.view.call_on_name(&self.inner_name, cb).expect("TODO")
    }
}

pub type MdView = MdViewT<Panel<ScrollView<NamedView<TextView>>>>;

pub struct MdViewT<T: View> {
    inner_name: String,
    view: T,
}

impl<T: View> ViewWrapper for MdViewT<T> {
    cursive::wrap_impl!(self.view: T);
}

impl MdView {
    pub fn new(name: Name) -> NamedView<Self> {
        let inner_name = name.to_string() + "_inner";
        let view = TextView::empty().with_name(&inner_name);
        let view = view.scrollable();
        let view = Panel::new(view);
        let view = MdViewT { view, inner_name };
        view.with_name(name)
    }

    /// Panics for now, to explore when result is None
    pub fn set_content<S>(&mut self, content: S)
    where
        S: Into<String>,
    {
        self.view
            .call_on_name(&self.inner_name, |tv: &mut TextView| {
                tv.set_content(markdown::parse(content))
            })
            .expect("unwrap failed in MdView.set_content")
    }
}

pub type FullLayout = FullLayoutT<PaddedView<LinearLayout>>;

pub struct FullLayoutT<T: View> {
    view: T,
    lr_margin: usize,
}

// TODO set child widths based on parent
impl ViewWrapper for FullLayoutT<PaddedView<LinearLayout>> {
    cursive::wrap_impl!(self.view: PaddedView<LinearLayout>);

    fn wrap_layout(&mut self, size: Vec2) {
        let margin = self.lr_margin;
        let horiz_xy = size.map_x(|x| x / 2 - margin);
        for ix in 0..2 {
            self.view
                .get_inner_mut()
                .get_child_mut(ix)
                .and_then(|pane| {
                    // Set top level horizontal constraints
                    pane.layout(horiz_xy);
                    // Then coerce the inner linear layouts
                    pane.downcast_mut()
                })
                // And get their children
                .and_then(|v: &mut LinearLayout| v.get_child_mut(0))
                // And set the inner vertical constraints
                .map(|v| v.layout(horiz_xy.map_y(|y| (ix + 1) * y / 3)));
        }
    }
}

impl FullLayout {
    pub fn new(
        lr_margin: usize,
        screen_size: Vec2,
        q_list: NamedView<ListView>,
        q_view: NamedView<MdView>,
        a_list: NamedView<ListView>,
        a_view: NamedView<MdView>,
    ) -> Self {
        let heuristic = 1;
        let x = SizeConstraint::Fixed(screen_size.x / 2 - lr_margin - heuristic);
        let y_list = SizeConstraint::AtMost(screen_size.y / 3);
        let y_view = SizeConstraint::Full; //AtLeast(2 * screen_size.y / 3);
        let view = LinearLayout::horizontal()
            .child(
                // TODO decide whats better, horizontal sizing on the outside,
                // or keeping both sizings on the 4 internal views
                LinearLayout::vertical()
                    .child(q_list.resized(x, y_list))
                    .child(q_view.resized(x, y_view))
                    .with_name("question-pane"), // TODO constants
            )
            .child(
                LinearLayout::vertical()
                    .child(a_list.resized(x, y_list))
                    .child(a_view.resized(x, y_view))
                    .with_name("answer-pane"),
            );
        let view = PaddedView::new(Margins::lrtb(lr_margin, lr_margin, 0, 0), view);
        FullLayoutT { view, lr_margin }
    }
}

fn add_vim_bindings<T: 'static>(
    view: NamedView<SelectView<T>>,
) -> OnEventView<NamedView<SelectView<T>>>
where
{
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            Some(EventResult::Consumed(Some(s.get_mut().select_up(1))))
        })
        .on_pre_event_inner('j', |s, _| {
            Some(EventResult::Consumed(Some(s.get_mut().select_down(1))))
        })
}

pub enum Layout {
    BothColumns,
    SingleColumn,
    FullScreen,
}

pub enum Mode {
    /// Akin to vim, keys are treated as commands
    Normal,
    /// Akin to vim, user is typing in bottom prompt
    Insert,
    // TODO if adding a search feature, that will be anther mode
}
