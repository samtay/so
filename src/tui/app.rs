#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::io::Write;
use tui::backend::CrosstermBackend;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout as TuiLayout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

use crate::error::Result;
use crate::stackexchange::{Answer, Question};
use crate::tui::enumerable::Enum;
use crate::tui::list;
// -----------------------------------------
// |question title list|answer preview list| 1/3
// -----------------------------------------
// |question body      |answer body        | 2/3
// -----------------------------------------
pub enum Layout {
    BothColumns,
    SingleColumn,
    FullScreen,
}

// Tab to cycle focus
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

// TODO take async questions
// TODO take the entire SE struct for future questions
pub fn run(qs: Vec<Question>) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    //terminal.draw(|mut f| ui::draw(&mut f, &mut app))?;
    terminal.draw(|mut f| {
        let chunks = TuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    // TODO this depends on app.ratio and app.layout
                    Constraint::Ratio(1, 2),
                    Constraint::Ratio(1, 2),
                ]
                .as_ref(),
            )
            .split(f.size());
        // TODO this depends on app.layout
        let question_pane = TuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
            .split(chunks[0]);
        let answer_pane = TuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
            .split(chunks[1]);
        let block = Block::default().title("Questions").borders(Borders::ALL);
        f.render_widget(block, question_pane[0]);
        let block = Block::default().title("Answers").borders(Borders::ALL);
        f.render_widget(block, answer_pane[0]);
        // for now, just text
        let t = qs[0].body.clone();
        let text = [Text::raw(t)];
        let p = Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(true);
        f.render_widget(p, question_pane[1]);
        let t = qs[0].answers[0].body.clone();
        let text = [Text::raw(t)];
        let p = Paragraph::new(text.iter())
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left)
            .wrap(true);
        f.render_widget(p, answer_pane[1]);
    })?;
    //disable_raw_mode()?;
    std::thread::sleep(std::time::Duration::from_millis(10000));
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}

//fmttext = termimad::FmtText::from(madskin, md, mwidth)
//// whose dispaly instance just calls
//for line in &self.lines {
//self.skin.write_fmt_line(f, line, self.width, false)?;
//writeln!(f)?;
//}

////OR directly with madskin
//skin.write_in_area_on(w: Writer, md: &str, area: &Area)

// lowest level
//skin.write_fmt_composite: actually applies styles to pieces of md text

// little higher
//skin.write_fmt_line: also handles lines such as table borders

// higher
//text.write_on: actually queues stuff up, cycling through its self.text.lines() and
//handling scrollbar

// TODO shift HJKL moves layout boundaries
// TODO hjkl to move focus? at least for lists..

// TODO should my se structs have &str instead of String?

// Space to cycle layout
// TODO query initial term size to init layout

impl Enum for Layout {
    fn to_enum(&self) -> u8 {
        match self {
            Layout::BothColumns => 0,
            Layout::SingleColumn => 1,
            Layout::FullScreen => 2,
        }
    }
    fn from_enum(i: u8) -> Self {
        match i % 3 {
            0 => Layout::BothColumns,
            1 => Layout::SingleColumn,
            _ => Layout::FullScreen,
        }
    }
}
