extern crate ncurses;

mod ui;
mod server;

const DEFAULT_ROOM: &str = "server";
const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn fill_room_window(room_window: ncurses::WINDOW, lines: &[String]) {
    fill_from_top_down(room_window, lines);
}

fn fill_from_top_down(window: ncurses::WINDOW, lines: &[String]) {
    let mut rows = 0;
    let mut cols = 0;
    ncurses::getmaxyx(window, &mut rows, &mut cols);
    rows -= 1;

    let to_print = std::cmp::min(lines.len(), rows as usize - 1);

    ncurses::wmove(window, 1, 1);
    for line in lines {
        let withnl = format!("{}\n", line);
        ncurses::mvwprintw(window,
                           1,
                           1,
                           &line);
    }
}

fn fill_chat_window(chat_window: ncurses::WINDOW, lines: &[String]) {
    fill_from_bottom_up(chat_window, lines);
}

fn fill_from_bottom_up(window: ncurses::WINDOW, lines: &[String]) {
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

    let mut curr_room = String::from(DEFAULT_ROOM);
    let mut room_msgs = server.get_messages(&curr_room)
        .expect("default room");
    fill_room_window(room_win, &[curr_room]);
    ncurses::wrefresh(room_win);

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
    let mut test_chat = vec![];
    loop {
        if buf.len() == 0 {
            ncurses::wmove(input_win, 1, 1);
        }
        
        match ui.readline(input_win, &mut buf) {
            Ok(_) => {
                test_chat.push(buf.clone());
                ui::clear_and_box(chat_win);
                fill_chat_window(chat_win, &test_chat);
                ncurses::wrefresh(chat_win);

                // Clean up the input window, clear the contents,
                // reset the buffer, and move the input cursor back
                // to its initial position.
                ncurses::wmove(input_win, 1, 1);
                ui::clear_and_box(input_win);
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
