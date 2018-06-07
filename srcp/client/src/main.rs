extern crate ncurses;

mod ui;

const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn main() {
    let mut ui = ui::Ui::new();
    ncurses::keypad(ncurses::stdscr(), true);
    ncurses::noecho();
    ncurses::cbreak();
    ncurses::halfdelay(1);

    let rows = ui.rows();
    let cols = ui.cols();

    let chat_win = ui.add_window(
        rows - INPUT_WINDOW_HEIGHT,
        cols - ROOM_WINDOW_WIDTH,
        0,
        ROOM_WINDOW_WIDTH).expect("chat window");

    let room_win = ui.add_window(
        rows - INPUT_WINDOW_HEIGHT,
        ROOM_WINDOW_WIDTH,
        0,
        0).expect("room window");

    let input_win = ui.add_window(
        INPUT_WINDOW_HEIGHT,
        cols,
        rows - INPUT_WINDOW_HEIGHT,
        0).expect("input window");

    let mut buf = String::new();
    loop {
        if buf.len() == 0 {
            ncurses::wmove(ui.win(input_win).expect("input win"), 1, 1);
        }
        
        match ui.readline(input_win, &mut buf) {
            Ok(_) => {
                ncurses::wprintw(ui.win(chat_win).unwrap(), &buf.clone());
                ncurses::wrefresh(ui.win(chat_win).unwrap());

                let inwin = ui.win(input_win).expect("input win");
                ncurses::wmove(inwin, 1, 1);
                ncurses::wclear(inwin);
                ncurses::box_(inwin, 0, 0);
                buf = String::new();
            },
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => (),
                    _ => break,
                }
            }
        }
    }
}
