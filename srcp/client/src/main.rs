extern crate ncurses;

use ncurses::*;

fn main() {
    initscr();
    noecho();

    let mut scr_width  = 0;
    let mut scr_height = 0;
    getmaxyx(stdscr(), &mut scr_width, &mut scr_height);
    

    getch();
    endwin();
}
