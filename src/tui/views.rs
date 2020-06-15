use cursive::event::{Callback, Event, EventResult};
use cursive::traits::{Finder, Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::view::{Margins, SizeConstraint, View, ViewWrapper};
use cursive::views::{
    LinearLayout, NamedView, OnEventView, PaddedView, Panel, ResizedView, ScrollView, SelectView,
    TextView,
};
use cursive::{Cursive, Vec2};
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use super::markdown;
use crate::error::Result;

pub const NAME_QUESTION_LIST: &str = "question_list";
pub const NAME_ANSWER_LIST: &str = "answer_list";
pub const NAME_QUESTION_VIEW: &str = "question_view";
pub const NAME_ANSWER_VIEW: &str = "answer_view";
pub const NAME_FULL_LAYOUT: &str = "full_layout";

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
pub type ListView =
    ListViewT<ResizedView<Panel<ScrollView<OnEventView<NamedView<SelectView<u32>>>>>>>;

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
        let view = view.resized(SizeConstraint::Free, SizeConstraint::Free);
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

    pub fn resize(&mut self, width: SizeConstraint, height: SizeConstraint) {
        self.view.set_constraints(width, height);
    }
}

pub type MdView = MdViewT<ResizedView<Panel<ScrollView<NamedView<TextView>>>>>;

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
        let view = view.resized(SizeConstraint::Free, SizeConstraint::Free);
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

    pub fn resize(&mut self, width: SizeConstraint, height: SizeConstraint) {
        self.view.set_constraints(width, height);
    }
}

pub type FullLayout = FullLayoutT<PaddedView<LinearLayout>>;

pub struct FullLayoutT<T: View> {
    view: T,
    invalidated: bool,
}

struct FullLayoutSizing {
    width: SizeConstraint,
    list_height: SizeConstraint,
    view_height: SizeConstraint,
}

// TODO set child widths based on parent
impl ViewWrapper for FullLayoutT<PaddedView<LinearLayout>> {
    cursive::wrap_impl!(self.view: PaddedView<LinearLayout>);

    // TODO what the actual fuck is wrong with this lifetime?
    //  cursive does this shit all over the place...
    // For now just issue a call_on_name like an asshat
    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        if let Event::WindowResize = event {
            println!("resize event thrown");
            self.invalidated = true;
        }
        self.view.on_event(event)
    }

    fn wrap_required_size(&mut self, req: Vec2) -> Vec2 {
        req
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.resize(size);
        self.invalidated = false;
        self.view.layout(size);
    }

    fn wrap_needs_relayout(&self) -> bool {
        self.invalidated || self.view.needs_relayout()
    }
}

impl FullLayout {
    pub fn new(
        lr_margin: usize,
        q_list: NamedView<ListView>,
        q_view: NamedView<MdView>,
        a_list: NamedView<ListView>,
        a_view: NamedView<MdView>,
    ) -> NamedView<Self> {
        let view = LinearLayout::horizontal()
            .child(
                // TODO decide whats better, horizontal sizing on the outside,
                // or keeping both sizings on the 4 internal views
                LinearLayout::vertical().child(q_list).child(q_view),
            )
            .child(LinearLayout::vertical().child(a_list).child(a_view));
        let view = PaddedView::new(Margins::lrtb(lr_margin, lr_margin, 0, 0), view);
        (FullLayoutT {
            view,
            invalidated: true,
        })
        .with_name(NAME_FULL_LAYOUT)
    }

    // public for now TODO remove
    pub fn resize(&mut self, size: Vec2) {
        let FullLayoutSizing {
            width,
            list_height,
            view_height,
        } = self.get_constraints(size);
        self.view
            .call_on_name(NAME_QUESTION_LIST, |v: &mut ListView| {
                v.resize(width, list_height)
            })
            .expect("TODO");
        self.view
            .call_on_name(NAME_ANSWER_LIST, |v: &mut ListView| {
                v.resize(width, list_height)
            })
            .expect("TODO");
        self.view
            .call_on_name(NAME_QUESTION_VIEW, |v: &mut MdView| {
                v.resize(width, view_height)
            })
            .expect("TODO");
        self.view
            .call_on_name(NAME_ANSWER_VIEW, |v: &mut MdView| {
                v.resize(width, view_height)
            })
            .expect("TODO");
        //.and_then(View::downcast_mut)
        //.map(View::downcast_mut)
        //.map(|v: &mut LinearLayout| {
        //println!("downcast successful!");
        //v.get_child_mut(0).and_then(View::downcast_mut).map(
        //|v: &mut ResizedView<NamedView<ListView>>| {
        //println!("LIST CONSTRAINTS SET");
        //v.set_constraints(width, list_height);
        //},
        //);
        //v.get_child_mut(1).and_then(View::downcast_mut).map(
        //|v: &mut ResizedView<NamedView<ListView>>| {
        //println!("VIEW CONSTRAINTS SET");
        //v.set_constraints(width, list_height);
        //},
        //)
        //});
    }

    fn get_constraints(&self, screen_size: Vec2) -> FullLayoutSizing {
        let heuristic = 1;
        let width = SizeConstraint::Fixed(screen_size.x / 2 - heuristic);
        let list_height = SizeConstraint::AtMost(screen_size.y / 3);
        let view_height = SizeConstraint::Full; //AtLeast(2 * screen_size.y / 3);
        println!(
            "list constraints: {} x {}",
            screen_size.x / 2 - heuristic,
            screen_size.y / 3
        );
        FullLayoutSizing {
            width,
            list_height,
            view_height,
        }
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
