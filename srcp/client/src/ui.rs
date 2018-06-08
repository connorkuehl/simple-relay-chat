use ::std;
use ::std::collections::HashMap;

use ::ncurses;

pub struct Ui {
    rows: usize,
    cols: usize,
    windows: Vec<ncurses::WINDOW>,
}

impl Ui {
    pub fn new() -> Ui {
        ncurses::initscr();
        let mut r = 0;
        let mut c = 0;
        ncurses::getmaxyx(ncurses::stdscr(), &mut r, &mut c);

        Ui {
            rows: r as usize,
            cols: c as usize,
            windows: vec![],
        }
    }

    pub fn add_window(
        &mut self,
        row: usize,
        col: usize,
        x: usize,
        y: usize) -> Result<ncurses::WINDOW, ()> {
        let w = ncurses::newwin(row as i32, col as i32, x as i32, y as i32);
        if w.is_null() {
            return Err(());
        }

        ncurses::box_(w, 0, 0);
        ncurses::wrefresh(w);

        self.windows.push(w);

        Ok(w)
    }

    pub fn readline(&self,
                    window: ncurses::WINDOW,
                    buf: &mut String) -> Result<(), std::io::Error> {
        let w = window;
        
        let ch = ncurses::wgetch(w);
        if ncurses::ERR != ch {
            match ch {
                ncurses::KEY_BACKSPACE => {
                    buf.pop();
                },
                _ => {
                    if let Some(ch) = std::char::from_u32(ch as u32) {
                        match ch {
                            '\n' => return Ok(()),
                            _ => {
                                buf.push(ch);
                                ncurses::wechochar(w, ch as u64);
                            },
                        }
                    }
                },
            }
        }
        
        Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "read timeout"))
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        for window in &self.windows {
            ncurses::delwin(*window);
        }
        ncurses::endwin();
    }
}

pub fn clear_and_box(window: ncurses::WINDOW) {
    ncurses::wclear(window);
    ncurses::box_(window, 0, 0);
}

pub fn fill_from_top_down(window: ncurses::WINDOW, lines: &[String]) {
    let mut rows = 0;
    let mut cols = 0;
    ncurses::getmaxyx(window, &mut rows, &mut cols);
    rows -= 1;

    let to_print = std::cmp::min(lines.len(), rows as usize - 1);

    ncurses::wmove(window, 1, 1);
    for i in 0..to_print {
        ncurses::mvwprintw(window, i as i32 + 1, 1, &lines[i]);
    }
}

pub fn fill_from_bottom_up(window: ncurses::WINDOW, lines: &[String]) {
    let mut rows = 0;
    let mut cols = 0;
    ncurses::getmaxyx(window, &mut rows, &mut cols);
    rows -= 1;

    let to_print = std::cmp::min(lines.len(), rows as usize - 1);

    ncurses::wmove(window, rows - 1, 1);
    for i in 0..to_print {
        ncurses::mvwprintw(window,
                rows - i as i32 - 1,
                1,
                &lines[lines.len() - i - 1]);
    }
}
