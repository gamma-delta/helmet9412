#![feature(try_blocks)]

mod audio;
mod canvases;
mod patterns;
mod scripting;

use audio::AudioHandler;
use canvases::{Canvas, HEIGHT, WIDTH};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{self, style, Color},
    terminal::{self, ClearType},
    QueueableCommand,
};
use patterns::PATTERNS;
use rhai::{exported_module, Engine, Scope, AST};

use std::{
    f64,
    io::{stdout, Write},
    thread,
    time::{Duration, Instant},
};
const PROMPT: &str = "(t,i,x,y,v,a)=>";

fn main() -> anyhow::Result<()> {
    let canvas = Canvas::new();
    let mut globals = Globals {
        canvas,
        audio: AudioHandler::new(),
    };
    let mode = &mut Mode::new(PATTERNS[0]);

    execute!(stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    'main: loop {
        let frame_start_time = Instant::now();
        match mode {
            Mode::Running(running) => {
                execute!(stdout(), cursor::Hide)?;

                // Don't lock and unlock it a billion times
                let volume = globals.audio.get_volume();

                let mut new_canvas = globals.canvas.clone();
                for x in 0..WIDTH {
                    for y in 0..HEIGHT {
                        let i = y * WIDTH + x;

                        let mut scope = Scope::new();
                        scope.push_constant("t", running.start_time.elapsed().as_secs_f64());
                        scope.push_constant("i", i as f64);
                        scope.push_constant("x", x as f64);
                        scope.push_constant("y", y as f64);
                        scope.push_constant("v", volume as f64);
                        scope.push_constant("a", globals.canvas.0[i]);

                        scope.push_constant("PI", f64::consts::PI);
                        scope.push_constant("TAU", f64::consts::TAU);
                        scope.push_constant("E", f64::consts::E);
                        scope.push_constant("PHI", 1.618_033_988_749_895);

                        let res = running.engine.eval_ast_with_scope(&mut scope, &running.ast);
                        drop(scope);
                        let res = match res {
                            Ok(it) => it,
                            Err(oh_no) => {
                                *mode = Mode::Editing(ModeEditing {
                                    error: Some(oh_no.to_string()),
                                    code: running.code.clone(),
                                });
                                continue 'main;
                            }
                        };
                        new_canvas.0[i] = res;
                    }
                }

                globals.canvas = new_canvas;

                globals.render_canvas()?;

                while event::poll(Duration::new(0, 0))? {
                    match event::read()? {
                        // ctrl+C cancels
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                        }) => break 'main,
                        // esc to edit code
                        Event::Key(KeyEvent {
                            code: KeyCode::Esc, ..
                        }) => {
                            *mode = Mode::Editing(ModeEditing {
                                error: None,
                                code: running.code.clone(),
                            });
                            continue 'main;
                        }
                        // number key to select a pattern
                        Event::Key(KeyEvent {
                            code: KeyCode::Char(digit @ '0'..='9'),
                            ..
                        }) => {
                            if let Some(code) = PATTERNS.get(digit.to_digit(10).unwrap() as usize) {
                                *mode = Mode::new(code);
                                globals.canvas = Canvas::new();
                                continue 'main;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Mode::Editing(ref mut editing) => {
                globals.render_canvas()?;

                let (width, height) = terminal::size()?;
                // If needed print the error message
                if let Some(ref error) = editing.error {
                    let lines_req = error.len() as u16 / width;
                    execute!(
                        stdout(),
                        cursor::MoveTo(0, height - 2 - lines_req),
                        terminal::EnableLineWrap,
                        style::PrintStyledContent(style(error).with(Color::DarkRed)),
                    )?;
                }

                let mut new_script = editing.code.clone();

                // Print the prompt and the remnants of the old code
                // this is the index of the character the cursor is to the left to
                // when it's 0, it's at the beginning.
                // the end is len-1
                let mut cursor_idx = new_script.len();
                let print_prompt = |code: &str, cursor_idx| -> anyhow::Result<()> {
                    execute!(
                        stdout(),
                        cursor::Hide,
                        cursor::MoveTo(0, height - 1),
                        terminal::Clear(ClearType::CurrentLine),
                        style::PrintStyledContent(style(PROMPT).with(Color::Grey)),
                        cursor::SavePosition,
                        style::PrintStyledContent(style(code).with(Color::White)),
                        cursor::RestorePosition,
                        cursor::MoveRight(cursor_idx as u16),
                        cursor::Show,
                    )?;
                    Ok(())
                };
                print_prompt(&new_script, cursor_idx)?;

                while let Event::Key(keys) = event::read()? {
                    match keys {
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers,
                        } if modifiers.intersects(KeyModifiers::CONTROL) => break 'main,
                        KeyEvent {
                            code: KeyCode::Esc, ..
                        } => {
                            // discard changes
                            execute!(stdout(), cursor::Hide, terminal::Clear(ClearType::All))?;
                            *mode = Mode::new(&editing.code);
                            continue 'main;
                        }
                        KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        } => {
                            break;
                        }

                        KeyEvent {
                            code: KeyCode::Char(c),
                            modifiers,
                        } => {
                            let c = if modifiers.intersects(KeyModifiers::SHIFT) {
                                c.to_ascii_uppercase()
                            } else {
                                c
                            };
                            // TODO: this won't do well with non-ASCII inputs.
                            let char_idx = new_script.char_indices().nth(cursor_idx);
                            match char_idx {
                                None => new_script.push(c),
                                Some((idx, _)) => new_script.insert(idx, c),
                            }
                            cursor_idx += 1;
                        }
                        KeyEvent {
                            code: KeyCode::Backspace,
                            modifiers,
                        } => {
                            if modifiers.intersects(KeyModifiers::SHIFT) {
                                new_script.clear();
                                cursor_idx = 0;
                            } else if cursor_idx > 0 {
                                new_script.remove(cursor_idx - 1);
                                cursor_idx -= 1;
                            }
                        }

                        KeyEvent {
                            code: KeyCode::Left,
                            ..
                        } => {
                            if cursor_idx > 0 {
                                cursor_idx -= 1;
                            }
                        }
                        KeyEvent {
                            code: KeyCode::Right,
                            ..
                        } => {
                            if cursor_idx < new_script.len() {
                                cursor_idx += 1;
                            }
                        }
                        KeyEvent {
                            code: KeyCode::Up, ..
                        } => {
                            cursor_idx = 0;
                        }
                        KeyEvent {
                            code: KeyCode::Down,
                            ..
                        } => {
                            cursor_idx = new_script.len();
                        }

                        _ => {}
                    }

                    print_prompt(&new_script, cursor_idx)?;
                }

                // clear prompt and error
                execute!(stdout(), cursor::Hide, terminal::Clear(ClearType::All))?;
                *mode = Mode::new(&new_script);
                globals.canvas = Canvas::new();
            }
        }

        let target_frametime = Duration::from_secs_f64(1.0 / 30.0);
        let elapsed_frametime = frame_start_time.elapsed();
        if let Some(delta) = target_frametime.checked_sub(elapsed_frametime) {
            thread::sleep(delta);
        }
    }

    stdout().queue(terminal::LeaveAlternateScreen)?;
    stdout().queue(cursor::Show)?;
    stdout().flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

struct Globals {
    canvas: Canvas,
    audio: AudioHandler,
}

enum Mode {
    Running(ModeRunning),
    Editing(ModeEditing),
}

impl Mode {
    /// Create a new Mode from the given string input.
    /// If it compiles, start `ModeRunning`, otherwise
    /// back to `ModeEditing`.
    ///
    /// This creates a new Engine and AST (and discards them if there's a problem.)
    fn new(code: &str) -> Self {
        let mut engine = Engine::new();
        let module = exported_module!(scripting::tixyva_utils);
        engine.register_global_module(module.into());

        match engine.compile(code) {
            Ok(ast) => Mode::Running(ModeRunning {
                start_time: Instant::now(),
                engine,
                ast,
                code: code.to_string(),
            }),
            Err(oh_no) => Mode::Editing(ModeEditing {
                error: Some(oh_no.to_string()),
                code: code.to_string(),
            }),
        }
    }
}

struct ModeRunning {
    start_time: Instant,
    engine: Engine,
    ast: AST,
    /// Original source code of this
    code: String,
}

struct ModeEditing {
    /// An error if anything bad happened
    error: Option<String>,
    /// The code previously printed here
    code: String,
}
