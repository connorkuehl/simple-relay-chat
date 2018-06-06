extern crate ncurses;

use std::net;
use std::sync::{Arc, Mutex};

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
    let mut input_row = scr_height - INPUT_WINDOW_HEIGHT as i32 - 1;
    let mut input_col = 0;
    keypad(input_window, true);

    let ncurse = Arc::new(Mutex::new(()));

    let lk = ncurse.clone();
    let ui = std::thread::spawn(move || {
        
    });

    let lk = ncurse.clone();
    let message = std::thread::spawn(move || {
        
    });

    ui.join().expect("ui thread panic");
    message.join().expect("message thread panic");

    getch();

    delwin(room_window);
    delwin(chat_window);
    delwin(input_window);
    endwin();
}
