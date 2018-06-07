extern crate ncurses;

mod ui;

const INPUT_WINDOW_HEIGHT: usize = 3;
const ROOM_WINDOW_WIDTH: usize = 16;

/*
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
 */

fn main() {
    let mut ui = ui::Ui::new();

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
        0);

    ncurses::getch();
}
