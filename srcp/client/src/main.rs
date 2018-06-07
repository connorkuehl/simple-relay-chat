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
        match ui.readline(input_win, &mut buf) {
            Ok(_) => {
                ncurses::wprintw(ui.win(chat_win).unwrap(), &buf.clone());
                ncurses::wrefresh(ui.win(chat_win).unwrap());
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
