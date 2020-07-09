#![allow(unused)]

use std::{
    error::Error,
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use linefeed::{Interface, ReadResult};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Paragraph, Row, Sparkline,
        Table, Tabs, Text,
    },
    Frame, Terminal,
};

enum State {
    LookingAtDm(crate::DMId),
    SingleMsg(crate::MsgId),
    // top msg we're looking at, in-order
    AllMsgList(crate::MsgId),
}

use std::sync::{Arc, Mutex};

struct App {
    should_quit: bool,
    s_lf: Arc<Mutex<crate::S_lf>>,
    states: Vec<State>,
    current_dm_view: crate::DMId,
    current_dm_list_view_top: crate::DMId,
}

impl App {}

fn draw<B: tui::backend::Backend>(f: &mut Frame<B>, view: &App) {
    f.render_widget(
        Paragraph::new(Some(Text::raw("welcome to s_lf.")).as_ref().into_iter()).wrap(true),
        f.size(),
    );
}

enum Control {
    Key,
    Line,
}

enum Event<I> {
    Key(I),
    LineRead(linefeed::reader::ReadResult),
    Tick,
}

pub fn main(s_lf: Arc<Mutex<crate::S_lf>>) -> Result<(), Box<dyn Error>> {
    let mut linereader = Interface::new("s_lf")?;
    linereader.set_prompt(":) ")?;

    let mut app = App {
        should_quit: false,
        s_lf,
        states: vec![],
        current_dm_view: 0, // todo: bubble this up and persist
        current_dm_list_view_top: 0,
    };

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup input handling
    let (event_tx, event_rx) = mpsc::channel();
    let (control_tx, control_rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let _: Result<(), std::io::Error> = (|| {
            let mut last_tick = Instant::now();
            // we alternate being invoking linefeed, which will read a line for us (eg if editing something).
            // and handling our own hotkey system
            loop {
                match control_rx.recv() {
                    Ok(Control::Key) => {
                        // poll for tick rate duration, if no events, sent tick event.
                        if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                            if let CEvent::Key(key) = event::read().unwrap() {
                                event_tx.send(Event::Key(key)).unwrap();
                            }
                        }
                    }
                    Ok(Control::Line) => {
                        event_tx.send(Event::LineRead(linereader.read_line()?));
                    }
                    Err(_) => panic!("wtf?"),
                }
                if last_tick.elapsed() >= tick_rate {
                    event_tx.send(Event::Tick).unwrap();
                    last_tick = Instant::now();
                }
            }
        })();
    });

    terminal.clear()?;

    loop {
        terminal.draw(|mut f| draw(&mut f, &app))?;
        let mut next_command = Control::Key;
        match event_rx.recv()? {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('i') => {
                    next_command = Control::Line;
                }
                _ => {}
            },
            Event::LineRead(ln) => {
                println!("read {:?}", ln);
            }
            Event::Tick => {}
        }
        control_tx.send(next_command);
    }

    Ok(())
}
