extern crate ncurses;

use ncurses::*;

const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn mkwin(lines: i32, cols: i32, row: i32, col: i32) -> ncurses::WINDOW {
    let w = newwin(lines, cols, row, col);
    box_(w, 0, 0);
    wrefresh(w);

    w
}

fn main() {
    initscr();
    
    let mut scr_width  = 0;
    let mut scr_height = 0;
    getmaxyx(stdscr(), &mut scr_height, &mut scr_width);

    refresh();

    let room_window = mkwin(scr_height - INPUT_WINDOW_HEIGHT as i32, ROOM_WINDOW_WIDTH as i32, 0, 0);

    let chat_window = mkwin(scr_height - INPUT_WINDOW_HEIGHT as i32, scr_width - ROOM_WINDOW_WIDTH as i32, 0, ROOM_WINDOW_WIDTH as i32);

    let input_window = mkwin(INPUT_WINDOW_HEIGHT as i32, scr_width, scr_height - INPUT_WINDOW_HEIGHT as i32, 0);
    let mut input_row = scr_height - INPUT_WINDOW_HEIGHT as i32 - 1;
    let mut input_col = 0;
    keypad(input_window, true);

    loop {
        wmove(input_window, 1, 1);
        wrefresh(input_window);

        let mut input = String::new();
        wgetstr(input_window, &mut input);
        wclear(input_window);
        box_(input_window, 0, 0);
        wrefresh(input_window);
        
        mvwaddstr(chat_window, 1, 1, &input);
        wrefresh(chat_window);
    }

    getch();

    delwin(room_window);
    delwin(chat_window);
    delwin(input_window);
    endwin();
}
