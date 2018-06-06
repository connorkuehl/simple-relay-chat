extern crate ncurses;

use std::net;
use std::collections::VecDeque;
use std::io::{Read, Write};

use ncurses::*;

const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

fn mkwin(lines: i32, cols: i32, row: i32, col: i32) -> ncurses::WINDOW {
    let w = newwin(lines, cols, row, col);
    box_(w, 0, 0);
    wrefresh(w);

    w
}

fn connect() -> std::io::Result<net::TcpStream> {
    printw("Connect to: ");
    let mut input = String::new();
    getstr(&mut input);

    net::TcpStream::connect(input)
}

fn identify(stream: &mut net::TcpStream) -> Result<(), ()> {
    clear();
    printw("Username: ");
    let mut input = String::new();
    getstr(&mut input);

    stream.write(format!("IDENTIFY {}", input).as_bytes()).expect("identify write");
    stream.flush().expect("identify flush");

    let mut buf = [0; 1024];
    match stream.read(&mut buf) {
        Ok(0) => {

        },
        Ok(bytes_read) => {
            let message = std::str::from_utf8(&buf).expect("from_utf8");

            if !message.starts_with("0") {
                return Err(());
            }
        },
        _ => {
            return Err(());
        }
    }

    Ok(())
}

fn main() {
    initscr();
    
    let mut scr_width  = 0;
    let mut scr_height = 0;
    getmaxyx(stdscr(), &mut scr_height, &mut scr_width);

    let mut stream = match connect() {
        Ok(s) => s,
        Err(e) => {
            let errormsg = format!("failed to connect: {}\nPress any key to quit.\n", e);
            mvprintw(1, 0, &errormsg);
            getch();
            endwin();
            std::process::exit(1);
        },
    };

    loop {
        if let Ok(_) = identify(&mut stream) {
            break;
        }

        mvprintw(1, 0, "username unavailable, try again\n");
    }

    let room_window = mkwin(scr_height - INPUT_WINDOW_HEIGHT as i32, ROOM_WINDOW_WIDTH as i32, 0, 0);

    let chat_window = mkwin(scr_height - INPUT_WINDOW_HEIGHT as i32, scr_width - ROOM_WINDOW_WIDTH as i32, 0, ROOM_WINDOW_WIDTH as i32);

    let input_window = mkwin(INPUT_WINDOW_HEIGHT as i32, scr_width, scr_height - INPUT_WINDOW_HEIGHT as i32, 0);

    let mut chat_x = 0;
    let mut chat_y = 0;

    getmaxyx(chat_window, &mut chat_y, &mut chat_x);
    
    let mut input_row = scr_height - INPUT_WINDOW_HEIGHT as i32 - 1;
    let mut input_col = 0;
    keypad(input_window, true);

    let mut messages = VecDeque::new();
    halfdelay(1);
    stream.set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .expect("set_read_timeout");

    let mut input = String::new();
    loop {
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(bytes_read) => {
                let unparsed = std::str::from_utf8(&buf).expect("from_utf8");
                let trimmed = unparsed[0..bytes_read].trim();

                let incoming: Vec<_> = trimmed.split("\n").collect();
                for msg in incoming {
                    if messages.len() as i32 >= scr_height - INPUT_WINDOW_HEIGHT as i32 - 1 as i32 {
                        messages.pop_front();
                    }
                    messages.push_back(String::from(msg));
                }

                wclear(chat_window);
                box_(chat_window, 0, 0);
                for i in 0..messages.len() {
                    mvwprintw(chat_window,
                              chat_y - INPUT_WINDOW_HEIGHT as i32 - i as i32 + 1 as i32,
                              1,
                              &messages[messages.len() - 1 - i]
                    );

                    wrefresh(chat_window);
                }
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => (),
                _ => break,
            },
        }

        let ch = wgetch(input_window);
        if ch != ERR && ch >= 0 {
            if let Some(ch) = std::char::from_u32(ch as u32) {
                match ch {
                    '\n' => {
                        stream.write(input.as_bytes()).expect("write");
                        stream.flush().expect("flush");
                        input = String::new();

                        wclear(input_window);
                        box_(input_window, 0, 0);
                        wmove(input_window, 1, 1);
                        wrefresh(input_window);
                    },
                    _ => input.push(ch),
                }
            }
        }
    }

    delwin(room_window);
    delwin(chat_window);
    delwin(input_window);
    endwin();
}
