extern crate common;
extern crate chrono;
extern crate ncurses;

mod ui;
mod server;

const DEFAULT_ROOM: &str = "server";
const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn fill_room_window(room_window: ncurses::WINDOW, lines: &[String]) {
    ui::fill_from_top_down(room_window, lines);
}

fn fill_chat_window(chat_window: ncurses::WINDOW, lines: &[String]) {
    ui::fill_from_bottom_up(chat_window, lines);
}

fn change_room(room_window: ncurses::WINDOW,
               curr: &mut String,
               server: &server::Server,
               up: bool) -> (String, Vec<String>) {
    let mut rooms = server.get_rooms();
    rooms.sort();

    let mut index = match rooms.iter().position(|r| r.eq(&curr.clone())) {
        Some(i) => i,
        None => 0,
    };

    let len = rooms.len();
    if len > 0 {
        if up {
            if index > 0 {
                index -= 1;
            }
        } else {
            if index < (len - 1) {
                index += 1;
            }
        }
    }
    
    let new_curr = rooms[index].clone();
    ui::clear_and_box(room_window);
    fill_room_window(room_window, &rooms);
    ncurses::wrefresh(room_window);

    let msgs = server.get_messages(&new_curr).expect("change room");

    (new_curr, msgs)
}

fn update_room_window(room_win: ncurses::WINDOW, rooms: &[String])
{
    ui::clear_and_box(room_win);
    fill_room_window(room_win, rooms);
    ncurses::wrefresh(room_win);
}

fn update_chat_room(win: ncurses::WINDOW, messages: &[String]) {
    ui::clear_and_box(win);
    fill_chat_window(win, messages);
    ncurses::wrefresh(win);

}

fn main() {
    let mut ui = ui::Ui::new();
    let mut server = server::Server::new("localhost:6667")
        .expect("failed to connect");

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
    
    ncurses::keypad(input_win, true);
    
    let mut curr_room = String::from(DEFAULT_ROOM);
    let mut rooms = server.get_rooms();
    fill_room_window(room_win, &rooms);
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
    loop {
        if buf.len() == 0 {
            ncurses::wmove(input_win, 1, 1);
        }
        
        match ui.readline(input_win, &mut buf) {
            Ok(key) => {
                match key {
                    ncurses::KEY_ENTER => {
                        // Dispatch message.
                        server.send(&buf.clone());

                        // Clean up the input window, clear the contents,
                        // reset the buffer, and move the input cursor back
                        // to its initial position.
                        ncurses::wmove(input_win, 1, 1);
                        ui::clear_and_box(input_win);
                        buf = String::new();
                    },
                    ncurses::KEY_UP => {
                        let (new_room, new_msgs) = change_room(
                            room_win,
                            &mut curr_room,
                            &server,
                            true);
                        curr_room = new_room;

                        update_chat_room(chat_win, &new_msgs);
                    },
                    ncurses::KEY_DOWN => {
                        let (new_room, new_msgs) = change_room(
                            room_win,
                            &mut curr_room,
                            &server,
                            false);
                        curr_room = new_room;

                        update_chat_room(chat_win, &new_msgs);
                    },
                    _ => (),
                }

            },
            Err(e) => {
                match e.kind() {
                    // This means a time out has occurred
                    std::io::ErrorKind::WouldBlock => (),
                    _ => (),
                }
            }
        }

        // Check server for new messages. Updates the chat and room
        // windows.
        if server.update().is_some() {
            let new_messages = server.get_messages(&curr_room).expect("curr room");
            update_chat_room(chat_win, &new_messages);
            
            rooms = server.get_rooms();
            update_room_window(room_win, &rooms);
        }
    }
}
