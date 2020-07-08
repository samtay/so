use cursive::event::{Callback, Event, EventResult, Key};
use cursive::traits::{Finder, Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::view::{Margins, SizeConstraint, View, ViewWrapper};
use cursive::views::{
    HideableView, LinearLayout, NamedView, PaddedView, Panel, ResizedView, ScrollView, SelectView,
    TextView,
};
use cursive::{Cursive, Vec2, XY};
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use super::markdown::Markdown;

pub const NAME_QUESTION_LIST: &str = "question_list";
pub const NAME_ANSWER_LIST: &str = "answer_list";
pub const NAME_QUESTION_VIEW: &str = "question_view";
pub const NAME_ANSWER_VIEW: &str = "answer_view";
pub const NAME_FULL_LAYOUT: &str = "full_layout";

// TODO this seems pointless; probably should be removed
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

trait Resize {
    fn set_width(&mut self, width: &SizeConstraint);
    fn set_height(&mut self, height: &SizeConstraint);
    fn resize(&mut self, width: &SizeConstraint, height: &SizeConstraint) {
        self.set_width(width);
        self.set_height(height);
    }
}

trait Hide {
    fn set_visible(&mut self, visible: bool);
    fn hide(&mut self) {
        self.set_visible(false);
    }
    fn unhide(&mut self) {
        self.set_visible(true);
    }
}

// TODO Copy select_view to to allow overriding selected style => reverse video
pub type ListView =
    ListViewT<HideableView<ResizedView<Panel<ScrollView<NamedView<SelectView<u32>>>>>>>;

pub struct ListViewT<T: View> {
    inner_name: String,
    view: T,
    force_take_focus: bool,
}

impl<T: View> ViewWrapper for ListViewT<T> {
    cursive::wrap_impl!(self.view: T);

    // In full screen mode we always take focus, even though currently hidden
    fn wrap_take_focus(&mut self, source: cursive::direction::Direction) -> bool {
        self.force_take_focus || self.view.take_focus(source)
    }

    // Always take arrow keys, its jarring to have them move pane focus
    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        let should_consume = match event {
            Event::Key(Key::Right)
            | Event::Key(Key::Left)
            | Event::Key(Key::Down)
            | Event::Key(Key::Up) => true,
            _ => false,
        };

        match self.view.on_event(event) {
            EventResult::Ignored if should_consume => EventResult::Consumed(None),
            event_result => event_result,
        }
    }
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
        let view = view.scrollable();
        let view = Panel::new(view).title(format!("{}", name));
        let view = view.resized(SizeConstraint::Free, SizeConstraint::Free);
        let view = HideableView::new(view);
        let view = ListViewT {
            view,
            inner_name,
            force_take_focus: false,
        };

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
        self.view
            .call_on_name(&self.inner_name, cb)
            .expect("Panic: Call on select_view failed")
    }

    pub fn set_take_focus(&mut self, take: bool) {
        self.force_take_focus = take;
    }
}

impl Resize for ListView {
    fn set_width(&mut self, width: &SizeConstraint) {
        self.view.get_inner_mut().set_width(*width);
    }
    fn set_height(&mut self, height: &SizeConstraint) {
        self.view.get_inner_mut().set_height(*height);
    }
}

impl Hide for ListView {
    fn set_visible(&mut self, visible: bool) {
        self.view.set_visible(visible);
    }
}

pub type MdView = MdViewT<HideableView<ResizedView<Panel<ScrollView<NamedView<TextView>>>>>>;

pub struct MdViewT<T: View> {
    inner_name: String,
    view: T,
    /// If the LayoutView is in full screen mode, MdView should always accept
    /// focus.
    force_take_focus: bool,
    title: String,
}

impl<T: View> ViewWrapper for MdViewT<T> {
    cursive::wrap_impl!(self.view: T);

    fn wrap_take_focus(&mut self, source: cursive::direction::Direction) -> bool {
        self.force_take_focus || self.view.take_focus(source)
    }

    // Always take arrow keys, its jarring to have them move pane focus
    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        let should_consume = match event {
            Event::Key(Key::Right)
            | Event::Key(Key::Left)
            | Event::Key(Key::Down)
            | Event::Key(Key::Up) => true,
            _ => false,
        };

        match self.view.on_event(event) {
            EventResult::Ignored if should_consume => EventResult::Consumed(None),
            event_result => event_result,
        }
    }
}

impl MdView {
    pub fn new(name: Name) -> NamedView<Self> {
        let inner_name = name.to_string() + "_inner";
        let view = TextView::empty().with_name(&inner_name);
        let view = view.scrollable();
        let view = Panel::new(view);
        let view = view.resized(SizeConstraint::Free, SizeConstraint::Free);
        let view = HideableView::new(view);
        let view = MdViewT {
            view,
            inner_name,
            title: name.to_string(),
            force_take_focus: false,
        };
        view.with_name(name)
    }

    /// Panics for now, to explore when result is None
    pub fn set_content(&mut self, content: &Markdown) {
        self.view
            .call_on_name(&self.inner_name, |tv: &mut TextView| {
                tv.set_content(content.clone())
            })
            .expect("unwrap failed in MdView.set_content")
    }

    pub fn show_title(&mut self) {
        self.view
            .get_inner_mut()
            .get_inner_mut()
            .set_title(&self.title);
    }

    pub fn hide_title(&mut self) {
        self.view.get_inner_mut().get_inner_mut().set_title("");
    }

    pub fn set_take_focus(&mut self, take: bool) {
        self.force_take_focus = take;
    }
}

impl Resize for MdView {
    fn set_width(&mut self, width: &SizeConstraint) {
        self.view.get_inner_mut().set_width(*width);
    }
    fn set_height(&mut self, height: &SizeConstraint) {
        self.view.get_inner_mut().set_height(*height);
    }
}

impl Hide for MdView {
    fn set_visible(&mut self, visible: bool) {
        self.view.set_visible(visible);
    }
}

pub struct LayoutView {
    view: PaddedView<LinearLayout>,
    layout: Layout,
    layout_invalidated: bool,
    size_invalidated: bool,
    last_size: Option<Vec2>,
}

struct LayoutViewSizing {
    width: SizeConstraint,
    list_height: SizeConstraint,
    view_height: SizeConstraint,
}

pub enum Layout {
    BothColumns,
    SingleColumn,
    FullScreen,
}

impl ViewWrapper for LayoutView {
    cursive::wrap_impl!(self.view: PaddedView<LinearLayout>);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        if let Event::Char(' ') = event {
            self.cycle_layout();
            self.layout_invalidated = true;
            return EventResult::Consumed(None);
        }

        self.view.on_event(event)
    }

    fn wrap_required_size(&mut self, req: Vec2) -> Vec2 {
        if self.last_size != Some(req) {
            self.size_invalidated = true;
            self.last_size = Some(req);
        }
        req
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.resize(size);
        self.relayout();
        self.size_invalidated = false;
        self.layout_invalidated = false;
        self.view.layout(size);
    }

    fn wrap_needs_relayout(&self) -> bool {
        self.layout_invalidated || self.size_invalidated || self.view.needs_relayout()
    }
}

impl LayoutView {
    pub fn new(
        lr_margin: usize,
        q_list: NamedView<ListView>,
        q_view: NamedView<MdView>,
        a_list: NamedView<ListView>,
        a_view: NamedView<MdView>,
    ) -> NamedView<Self> {
        let view = LinearLayout::horizontal()
            .child(LinearLayout::vertical().child(q_list).child(q_view))
            .child(LinearLayout::vertical().child(a_list).child(a_view));
        let view = PaddedView::new(Margins::lrtb(lr_margin, lr_margin, 0, 0), view);
        (LayoutView {
            view,
            layout_invalidated: true,
            size_invalidated: true,
            last_size: None,
            layout: Layout::BothColumns,
        })
        .with_name(NAME_FULL_LAYOUT)
    }

    fn get_constraints(&self, screen_size: Vec2) -> LayoutViewSizing {
        let heuristic = 1;
        let width = SizeConstraint::Fixed(screen_size.x / 2 - heuristic);
        let list_height = SizeConstraint::AtMost(screen_size.y / 3);
        let view_height = SizeConstraint::Full;

        LayoutViewSizing {
            width,
            list_height,
            view_height,
        }
    }

    fn resize(&mut self, size: Vec2) {
        let LayoutViewSizing {
            width,
            list_height,
            view_height,
        } = self.get_constraints(size);
        self.call_on_list_views(move |v| v.resize(&width, &list_height));
        self.call_on_md_views(move |v| v.resize(&width, &view_height));
    }

    // TODO separate out resizing | relayout | refocus; these are separate
    // concerns and should have their own methods of invalidation
    fn relayout(&mut self) {
        match self.layout {
            Layout::BothColumns => {
                self.call_on_list_views(|v| {
                    v.set_take_focus(false);
                    v.unhide();
                });
                self.call_on_md_views(|v| {
                    v.set_take_focus(false);
                    v.unhide();
                    v.hide_title();
                });
            }
            Layout::SingleColumn => {
                self.call_on_md_views(|v| {
                    v.hide();
                    v.set_width(&SizeConstraint::Full);
                });
                self.call_on_list_views(|v| {
                    v.hide();
                    v.set_width(&SizeConstraint::Full);
                    v.set_take_focus(true);
                });
                let name = Self::xy_to_name(self.get_focused_index());
                if name == NAME_QUESTION_LIST || name == NAME_QUESTION_VIEW {
                    self.view
                        .call_on_name(NAME_QUESTION_LIST, |v: &mut ListView| {
                            v.unhide();
                        });
                    self.view
                        .call_on_name(NAME_QUESTION_VIEW, |v: &mut MdView| {
                            v.unhide();
                        });
                } else {
                    self.view
                        .call_on_name(NAME_ANSWER_LIST, |v: &mut ListView| {
                            v.unhide();
                        });
                    self.view.call_on_name(NAME_ANSWER_VIEW, |v: &mut MdView| {
                        v.unhide();
                    });
                }
            }
            Layout::FullScreen => {
                self.call_on_md_views(|v| {
                    v.show_title();
                    v.hide();
                    v.set_take_focus(true);
                    v.resize(&SizeConstraint::Full, &SizeConstraint::Full);
                });
                self.call_on_list_views(|v| {
                    v.hide();
                    v.resize(&SizeConstraint::Full, &SizeConstraint::Full);
                });
                let name = Self::xy_to_name(self.get_focused_index());
                if name == NAME_QUESTION_LIST || name == NAME_ANSWER_LIST {
                    self.view.call_on_name(name, |v: &mut ListView| {
                        v.unhide();
                    });
                } else {
                    self.view.call_on_name(name, |v: &mut MdView| {
                        v.unhide();
                    });
                }
            }
        }
    }

    fn cycle_layout(&mut self) {
        self.layout = match self.layout {
            Layout::BothColumns => Layout::SingleColumn,
            Layout::SingleColumn => Layout::FullScreen,
            Layout::FullScreen => Layout::BothColumns,
        }
    }

    fn call_on_list_views<F>(&mut self, f: F)
    where
        F: Fn(&mut ListView) + 'static,
    {
        let f: Rc<dyn Fn(&mut ListView)> = Rc::new(move |v| f(v));
        self.view
            .call_on_name(NAME_QUESTION_LIST, &*f)
            .expect("Panic: call on question list failed");
        self.view
            .call_on_name(NAME_ANSWER_LIST, &*f)
            .expect("Panic: call on answer list failed");
    }

    fn call_on_md_views<F>(&mut self, f: F)
    where
        F: Fn(&mut MdView) + 'static,
    {
        let f: Rc<dyn Fn(&mut MdView)> = Rc::new(move |v| f(v));
        self.view
            .call_on_name(NAME_QUESTION_VIEW, &*f)
            .expect("Panic: call on question view failed");
        self.view
            .call_on_name(NAME_ANSWER_VIEW, &*f)
            .expect("Panic: call on answer view failed");
    }

    fn get_focused_index(&self) -> Vec2 {
        let top = self.view.get_inner();
        let x = top.get_focus_index();
        let inner = top
            .get_child(x)
            .unwrap()
            .downcast_ref::<LinearLayout>()
            .unwrap();
        let y = inner.get_focus_index();
        XY { x, y }
    }

    fn xy_to_name(xy: Vec2) -> &'static str {
        match xy {
            XY { x: 0, y: 0 } => NAME_QUESTION_LIST,
            XY { x: 0, y: 1 } => NAME_QUESTION_VIEW,
            XY { x: 1, y: 0 } => NAME_ANSWER_LIST,
            _ => NAME_ANSWER_VIEW,
        }
    }
}

/// Note that as it stands, this is very intrusive, and disallows the idea of
/// having an inner EditView. If more nuance is needed later, then it will keep
/// track of a `Mode = Insert | Command`
pub struct VimBindingsView<T: View> {
    last_event: Option<Event>,
    view: T,
}

impl<T: View> ViewWrapper for VimBindingsView<T> {
    cursive::wrap_impl!(self.view: T);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('g') => {
                if let Some(Event::Char('g')) = self.last_event {
                    self.last_event = None;
                    return self.view.on_event(Event::Key(Key::Home));
                }
                self.last_event = Some(Event::Char('g'));
            }
            Event::Char('Z') => {
                if let Some(Event::Char('Z')) = self.last_event {
                    self.last_event = None;
                    return EventResult::with_cb(|s| s.quit());
                }
                self.last_event = Some(Event::Char('Z'));
            }
            _ => self.last_event = None,
        }
        match event {
            Event::Char('q') => {
                return EventResult::with_cb(|s| s.quit());
            }
            Event::Char('h') => {
                return self.view.on_event(Event::Key(Key::Left));
            }
            Event::Char('j') => {
                return self.view.on_event(Event::Key(Key::Down));
            }
            Event::Char('k') => {
                return self.view.on_event(Event::Key(Key::Up));
            }
            Event::Char('l') => {
                return self.view.on_event(Event::Key(Key::Right));
            }
            Event::CtrlChar('d') => {
                return self.view.on_event(Event::Key(Key::PageDown));
            }
            Event::CtrlChar('f') => {
                // Double page down
                let res = self.view.on_event(Event::Key(Key::PageDown));
                self.view.on_event(Event::Key(Key::PageDown));
                // Return whether or not initial page down was consumed
                return res;
            }
            Event::CtrlChar('u') => {
                return self.view.on_event(Event::Key(Key::PageUp));
            }
            Event::CtrlChar('b') => {
                // Double page up
                let res = self.view.on_event(Event::Key(Key::PageUp));
                self.view.on_event(Event::Key(Key::PageUp));
                return res;
            }
            Event::Char('G') => {
                return self.view.on_event(Event::Key(Key::End));
            }
            _ => (),
        }
        self.view.on_event(event)
    }
}

impl<T: View> VimBindingsView<T> {
    fn new(view: T) -> Self {
        VimBindingsView {
            view,
            last_event: None,
        }
    }
}

pub trait Vimable: View + Sized {
    fn add_vim_bindings(self) -> VimBindingsView<Self> {
        VimBindingsView::new(self)
    }
}

impl<T: View> Vimable for T {}
