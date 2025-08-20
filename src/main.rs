use color_eyre::owo_colors::OwoColorize;
use image::{DynamicImage, GenericImageView, ImageReader};
use std::{
    cell::RefCell,
    io::{self, Bytes, Cursor},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};
use tachyonfx::Duration;
use web_sys::HtmlImageElement;

use ratzilla::{
    ratatui::{
        layout::{
            Alignment, Constraint,
            Direction::{self, Horizontal},
            Layout,
        },
        style::{Color, Modifier, Style, Stylize},
        text::{self, Line, Span, ToLine},
        widgets::{
            canvas::{Canvas, Context, Map, MapResolution, Shape},
            Block, BorderType, List, ListState, Paragraph, Widget, Wrap,
        },
        Frame, Terminal,
    },
    utils::call_js_function,
};
use tachyonfx::{
    fx, Effect, EffectRenderer, EffectTimer, Interpolation, Motion, Shader, SimpleRng,
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    CanvasBackend,
    DomBackend,
    // WebGl2Backend,
    WebRenderer,
};

mod colors;
mod macros;
use colors::ColourTheme;

// TODO: Include a few more of these for different screen sizes
// This is used later on as *banner art*
static TITLE_ART: &str = r"
██╗    ██╗██████╗ ██████╗███╗   ███████████████████████╗███████╗
██║    ████╔═══████╔═══██████╗  ██╚══██╔══██╔════██╔══████╔════╝
██║ █╗ ████║   ████║   ████╔██╗ ██║  ██║  █████╗ ██████╔███████╗
██║███╗████║   ████║   ████║╚██╗██║  ██║  ██╔══╝ ██╔══██╚════██║
╚███╔███╔╚██████╔╚██████╔██║ ╚████║  ██║  █████████║  █████████║
 ╚══╝╚══╝ ╚═════╝ ╚═════╝╚═╝  ╚═══╝  ╚═╝  ╚══════╚═╝  ╚═╚══════╝";

static HEADSHOT: &[u8; 883046] = include_bytes!("../static/smallest.png");

/// Entry point for code, setup stuff and pass it off to ratzilla functions.
///
/// # Panics
///
/// Panics if the app's mutex is poisoned.
/// at this point some app function will have panic itself so we should be attempting to exit anyway
///
///
/// # Errors
///
/// This function will return an error if backend / terminal initialisation fails
fn main() -> io::Result<()> {
    // We should have a DOM backend version for accessibility
    let backend = CanvasBackend::new()?;
    let terminal = Terminal::new(backend)?;

    // Note sure why Arc is suggesting Mutex<App> isn't Send + Sync ( clippy even suggests wrapping it in a Mutex!)
    // We do this so the on_key_event and draw_web functions can both capture and mutate the app when needed
    let state: Arc<Mutex<App>> = Arc::new(Mutex::new(App::default()));
    // This channel handles sending messages for when to change colour, this allows us to hide the colour change in the middle of an animation making it much smoother
    let (tx, rx) = mpsc::channel();
    {
        let mut mod_state = state.lock().unwrap();
        mod_state.theme.borrow_mut().switch_colour(); // quickly switch colours at the start so we are on the first theme
        mod_state.main_state_animations.tx = Some(tx);
        mod_state.rx = Some(rx);
    }

    let event_state = Arc::clone(&state);
    terminal.on_key_event(move |key_event| {
        event_state.lock().unwrap().handle_events(&key_event);
    });

    let render_state = Arc::clone(&state);
    terminal.draw_web(move |frame| {
        render_state.lock().unwrap().render(frame);
    });

    Ok(())
}

/// App is the general struct which holds all the state / data about the site
///
/// Each state has animations and its own struct to store data
#[derive(Default)]
struct App {
    theme: RefCell<ColourTheme>,
    tab: Tabs,
    tabs_state: Arc<Mutex<ListState>>,
    main_state: MainState,
    main_state_animations: MainAnimationState,
    rng: SimpleRng,
    rx: Option<Receiver<ColourEvent>>,
}

// Enum for storing what tab we are looking at
#[derive(Copy, Clone)]
enum Tabs {
    Main,
    Blog,
}

impl Default for Tabs {
    fn default() -> Self {
        Self::Main
    }
}

// Enum for sending when we want to switch colour schemes
#[derive(Clone, Debug)]
enum ColourEvent {
    Switch,
}

// Storing any state data from the main page
#[derive(Default)]
struct MainState {
    links_state: Arc<Mutex<ListState>>,
}

// Storing Effect data for all the animations on the main screen
struct MainAnimationState {
    tabs_effect: Effect,
    title_effect: Effect,
    mini_about_effect: Effect,
    links_effect: Effect,
    about_effect: Effect,
    headshot_effect: Effect,
    help_effect: Effect,
    tx: Option<Sender<ColourEvent>>,
}

impl MainAnimationState {
    /// Whenever we switch colour themes we want to slide out the old colours to a neutral background, then slide the new theme in
    /// One animation needs to trigger sending a message to tx
    /// the rest have slightly random offsets to make it all a little less uniform
    fn create_fresh_animations(&mut self, bg_1: Color, rng: &mut SimpleRng) {
        self.title_effect = slide_in_and_out_disp!(
            0,
            bg_1,
            self.tx.as_ref().unwrap().clone(),
            ColourEvent::Switch
        );
        self.mini_about_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
        self.links_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
        self.about_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
        self.headshot_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
        self.tabs_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
        self.help_effect = slide_in_and_out!(rng.gen() % 100, bg_1);
    }
}

impl Default for MainAnimationState {
    /// Initial slide in animations for all of the cells
    fn default() -> Self {
        MainAnimationState {
            tabs_effect: fx::slide_in(
                Motion::DownToUp,
                10,
                1,
                Color::from_u32(0x0010_1010),
                EffectTimer::from_ms(500, Interpolation::Linear),
            ),
            title_effect: fx::slide_in(
                Motion::DownToUp,
                10,
                1,
                Color::from_u32(0x0010_1010),
                EffectTimer::from_ms(500, Interpolation::Linear),
            ),
            mini_about_effect: fx::prolong_start(
                100,
                fx::slide_in(
                    Motion::DownToUp,
                    10,
                    1,
                    Color::from_u32(0x0010_1010),
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            links_effect: fx::prolong_start(
                150,
                fx::slide_in(
                    Motion::DownToUp,
                    10,
                    1,
                    Color::from_u32(0x0010_1010),
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            about_effect: fx::prolong_start(
                90,
                fx::slide_in(
                    Motion::DownToUp,
                    10,
                    1,
                    Color::from_u32(0x0010_1010),
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            headshot_effect: fx::prolong_start(
                20,
                fx::slide_in(
                    Motion::DownToUp,
                    10,
                    1,
                    Color::from_u32(0x0010_1010),
                    EffectTimer::from_ms(500, Interpolation::Linear),
                ),
            ),
            help_effect: fx::slide_in(
                Motion::DownToUp,
                10,
                1,
                Color::from_u32(0x0010_1010),
                EffectTimer::from_ms(500, Interpolation::Linear),
            ),
            tx: Option::None,
        }
    }
}

impl App {
    // What we do each frame, here we want to
    fn render(&mut self, frame: &mut Frame) {
        if self.rx.as_ref().unwrap().try_recv().is_ok() {
            self.theme.borrow_mut().switch_colour();
        }
        match self.tab {
            Tabs::Main => self.render_main(frame),
            Tabs::Blog => todo!(),
        }
    }

    fn render_main(&mut self, frame: &mut Frame<'_>) {
        let o_total_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(3),
                Constraint::Fill(2),
                Constraint::Max(2),
            ])
            .split(frame.area());
        let o0_layout = Layout::default()
            .direction(Horizontal)
            .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(o_total_layout[1]);
        let o1_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Fill(10)])
            .split(o0_layout[0]);
        let o2_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(o1_layout[0]);
        let o3_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(25)])
            .split(o0_layout[1]);
        let o4_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(40)])
            .split(o3_layout[1]);

        let help_bar = self.gen_help_bar();
        let tabs_bar = self.gen_nav_bar();
        let title = self.gen_title();
        let mini_about = self.gen_mini_about();
        let links = self.gen_links();
        let about = self.gen_about();
        let headshot = self.canvas(HEADSHOT, "hey! that's me", [100.0, 500.0], [100.0, 750.0]);
        let empty = Block::new().bg(self.theme.borrow().color_bg);

        let mut links_state = self
            .main_state
            .links_state
            .lock()
            .expect("List state poisoned, Someone is messing with the DOM? ");

        let mut tabs_state = self
            .tabs_state
            .lock()
            .expect("Tabs state poisoned, Something is messing with the DOM");

        frame.render_stateful_widget(tabs_bar, o_total_layout[0], &mut tabs_state);
        frame.render_widget(help_bar, frame.area());
        frame.render_widget(title, o2_layout[0]);
        frame.render_widget(mini_about, o2_layout[1]);
        frame.render_stateful_widget(links, o1_layout[1], &mut links_state);
        frame.render_widget(about, o3_layout[0]);
        frame.render_widget(headshot, o4_layout[1]);
        frame.render_widget(empty, o4_layout[0]);
        animate!(
            (
                (self.main_state_animations.title_effect, o2_layout[0]),
                (self.main_state_animations.mini_about_effect, o2_layout[1]),
                (self.main_state_animations.links_effect, o1_layout[1]),
                (self.main_state_animations.about_effect, o3_layout[0]),
                (self.main_state_animations.headshot_effect, o3_layout[1]),
                (self.main_state_animations.tabs_effect, o_total_layout[0]),
                (self.main_state_animations.help_effect, frame.area())
            ),
            frame,
            7
        );
    }

    fn canvas<'a, const S: usize>(
        &'a self,
        image: &'a [u8; S],
        name: &'a str,
        width: [f64; 2],
        height: [f64; 2],
    ) -> impl Widget + 'a {
        Canvas::default()
            .block(
                Block::bordered()
                    .title(name)
                    .fg(self.theme.borrow().color_fg)
                    .bg(self.theme.borrow().color_bg),
            )
            .marker(ratzilla::ratatui::symbols::Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&ImageShape::new(
                    image,
                    self.theme.borrow().color_bg,
                    ColourType::Grey,
                ));
            })
            .x_bounds(width)
            .y_bounds(height)
    }

    fn handle_events(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => self
                .main_state
                .links_state
                .lock()
                .expect("List state poisoned, someone is messing with the DOM?")
                .select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self
                .main_state
                .links_state
                .lock()
                .expect("List state is poisoned, someone is messing with the DOM?")
                .select_next(),
            KeyCode::Char('W') => self.cycle_colour(),
            KeyCode::Enter => {
                if let Some(v) = self.main_state.links_state.lock().unwrap().selected() {
                    let url = match v {
                        0 => "https://github.com/woonters",
                        1 => "https://youtube.com/@woonters",
                        2 => "https://twitter.com/woonters",
                        _ => "https://github.com/woonters",
                    };
                    let _ = call_js_function("open", vec![url]);
                }
            }
            _ => {}
        }
    }

    fn cycle_colour(&mut self) {
        let bg_1_old = self.theme.borrow().color_bg;
        self.main_state_animations
            .create_fresh_animations(bg_1_old, &mut self.rng);
    }

    fn gen_instructions(&'_ self) -> Line<'_> {
        Line::from(vec![
            " Switch colour theme ".into(),
            "<W>".fg(self.theme.borrow().color_fg_alt).bold(),
            " Next List Item ".into(),
            "<j>".fg(self.theme.borrow().color_fg_alt).bold(),
            " Previous List Item".into(),
            "<k>".fg(self.theme.borrow().color_fg_alt).bold(),
            " Select List Item ".into(),
            "<enter>".fg(self.theme.borrow().color_fg_alt).bold(),
        ])
    }

    fn gen_help_bar(&self) -> Block {
        Block::bordered()
            .title_bottom(self.gen_instructions())
            .fg(self.theme.borrow().color_fg)
            .bg(self.theme.borrow().color_bg)
    }

    fn gen_title(&self) -> Paragraph<'_> {
        let title_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .fg(self.theme.borrow().color_fg)
            .title("WhoamI?");

        Paragraph::new(TITLE_ART)
            .block(title_block)
            .fg(self.theme.borrow().color_fg_alt)
            .bg(self.theme.borrow().color_bg)
            .centered()
    }

    fn gen_mini_about(&self) -> Paragraph<'_> {
        let mini_about_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .title("WhatAmI??");
        let text = text::Span::raw("Hi, I'm Jemma (She / Her), come look at my silly things :p");

        Paragraph::new(text)
            .block(mini_about_block)
            .fg(self.theme.borrow().color_fg)
            .bg(self.theme.borrow().color_bg)
            .wrap(Wrap { trim: true })
            .centered()
    }

    fn gen_nav_bar(&self) -> List<'_> {
        let nav_block = Block::bordered()
            .title("Navigation")
            .border_type(BorderType::Rounded);
        let tabs_list = vec!["Main", "Blog"];
        List::new(tabs_list)
            .block(nav_block)
            .fg(self.theme.borrow().color_fg)
            .bg(self.theme.borrow().color_bg)
            .highlight_symbol(">")
            .repeat_highlight_symbol(true)
    }

    fn gen_links(&self) -> List<'_> {
        let links_block = Block::bordered()
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .title("Links");
        let links_list = vec!["Github", "Youtube", "Twitter"];
        List::new(links_list)
            .block(links_block)
            .fg(self.theme.borrow().color_fg)
            .bg(self.theme.borrow().color_bg)
            .highlight_symbol(">")
            .repeat_highlight_symbol(true)
    }

    fn gen_about(&self) -> Paragraph<'_> {
        let about_block = Block::bordered()
            .title_alignment(Alignment::Left)
            .border_type(BorderType::Rounded)
            .fg(self.theme.borrow().color_fg)
            .title("About");
        let about_text = vec![
            text::Line::from(
                vec![ Span::from("I'm "),
                    Span::styled("Jemma",Style::default().fg(self.theme.borrow().color_fg)),
                    Span::from(", I write code, make bad music, "),
                 Span::styled("animate",Style::default().add_modifier(Modifier::BOLD))
                 ,Span::from(" ,and generally get distracted.")]),
          text::Line::from("Look out for cool little things I make, they might interest you."),
          text::Line::from(
              vec![
                  Span::from("I generally use "),
                  Span::styled("Rust", Style::default().fg(self.theme.borrow().color_fg)),
                  Span::from(" for most of my most interesting projects (maybe you should read about them on my "),
                  Span::styled("blog", Style::default().fg(self.theme.borrow().color_fg)),
                  Span::from(")"),
              ]
          ),
          text::Line::from(
              vec![
                  Span::from("On other occasions I use "),
                  Span::styled("Python", Style::default().fg(self.theme.borrow().color_fg)),
                  Span::from(". But coding isn't my only hobby, I've recently been making music, 3d modeling, animating and writing."),
              ]
          ),
        ];
        Paragraph::new(about_text)
            .block(about_block)
            .fg(self.theme.borrow().color_fg_alt)
            .bg(self.theme.borrow().color_bg)
            .centered()
            .wrap(ratzilla::ratatui::widgets::Wrap { trim: true })
    }
}

// Future work on a renderer for full colour images (should be trivial)
enum ColourType {
    Full,
    Grey,
}

// What we use for drawing images
struct ImageShape {
    image_buffer: DynamicImage,
    tint_colour: Color,
    colour_type: ColourType,
    max: u8,
}

impl ImageShape {
    fn new<const S: usize>(image: &[u8; S], tint_colour: Color, colour_type: ColourType) -> Self {
        // Read the image (it'll be a byte array stored in the binary atm) move this over to web_sys assets in the static folder when possible
        // but doing this might make it a paint as you will need to draw the image to an invisible buffer before you are able to get at the pixles
        // atleast from how the documentation looks ughhhhh
        let img = ImageReader::new(Cursor::new(image))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .flipv(); // we flipv because for whatever reason the image is upside-down
        let max = img // to make the grey scaled image better we neeed to adjust the image to the max brightness (this gets something close to that)
            .pixels()
            .max_by(|x, y| x.2 .0.iter().sum::<u8>().cmp(&y.2 .0.iter().sum::<u8>()))
            .unwrap()
            .2
             .0
            .iter()
            .sum::<u8>()
            / 4; // really we should be doing some luminance shit here
        Self {
            image_buffer: img,
            tint_colour,
            colour_type,
            max,
        }
    }
}

impl Shape for ImageShape {
    fn draw(&self, painter: &mut ratzilla::ratatui::widgets::canvas::Painter) {
        // read the image as luma8 and then start writing each pixle to the canvas
        let binding = self.image_buffer.to_luma8();
        let pixles = binding.pixels();
        let w = binding.width() as usize;
        pixles.enumerate().for_each(|(i, p)| {
            let x = i % w;
            let y = i / w;
            if let Some((x, y)) = painter.get_point(x as f64, y as f64) {
                let h = (p.0[0] as f64) / self.max as f64;
                match self.tint_colour {
                    Color::Rgb(r, g, b) => painter.paint(
                        x,
                        y,
                        Color::Rgb(
                            (r as f64 * h) as u8,
                            (g as f64 * h) as u8,
                            (b as f64 * h) as u8,
                        ),
                    ),
                    _ => unimplemented!("We currently expect the colour passed to the iamge render to be a Color::Rgb"),
                }
            }
        });
    }
}
