use ::std::collections::HashMap;

use ::ncurses;

type Wid = usize;

pub struct Ui {
    rows: usize,
    cols: usize,
    nextwid: Wid,
    windows: HashMap<Wid, ncurses::WINDOW>,
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
            nextwid: 1,
            windows: HashMap::new(),
        }
    }

    pub fn add_window(
        &mut self,
        row: usize,
        col: usize,
        x: usize,
        y: usize) -> Result<Wid, ()> {
        let w = ncurses::newwin(row as i32, col as i32, x as i32, y as i32);
        if w.is_null() {
            return Err(());
        }

        let wid = self.nextwid;

        if self.windows.insert(wid, w).is_some() {
            return Err(());
        }
        
        self.nextwid += 1;

        ncurses::box_(w, 0, 0);
        ncurses::wrefresh(w);

        Ok(wid)
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
        for window in self.windows.values() {
            ncurses::delwin(*window);
        }
        ncurses::endwin();
    }
}
