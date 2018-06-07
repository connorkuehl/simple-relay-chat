extern crate ncurses;

mod ui;
mod server;

const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn main() {
    let mut ui = ui::Ui::new();
    let mut server = server::Server::new("localhost:6667")
        .expect("failed to connect");
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

    // Input update loop - a single-threaded compromise
    // for a simple client implementation.
    //
    // User input
    // 
    // User input is buffered, but on timeouts. The buffer
    // will be updated when they finally do interact with
    // the keyboard. When the user hits 'enter', the buffer
    // is committed and their input is ready for parsing
    // and eventually dispatching to the relay server.
    //
    // Server updates
    //
    // The socket is checked with timeouts. If there is data
    // waiting, the client will parse the lines and commit
    // them to the appropriate data structures.
    let mut buf = String::new();
    loop {
        if buf.len() == 0 {
            ncurses::wmove(ui.win(input_win).expect("input win"), 1, 1);
        }
        
        match ui.readline(input_win, &mut buf) {
            Ok(_) => {
                let chatwin = ui.win(chat_win).expect("chat win");

                // TODO: remove me after testing...
                ncurses::wprintw(chatwin, &buf.clone());
                ncurses::wrefresh(chatwin);

                // Clean up the input window, clear the contents,
                // reset the buffer, and move the input cursor back
                // to its initial position.
                let inwin = ui.win(input_win).expect("input win");
                ncurses::wmove(inwin, 1, 1);
                ncurses::wclear(inwin);
                ncurses::box_(inwin, 0, 0);
                buf = String::new();
            },
            Err(e) => {
                match e.kind() {
                    // This means a time out has occurred
                    std::io::ErrorKind::WouldBlock => (),
                    // TODO: this is an actual error.
                    _ => break,
                }
            }
        }
    }
}
