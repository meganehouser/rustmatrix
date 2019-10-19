use cursive::Cursive;
use cursive::views::{Canvas, Dialog, SizedView, TextView, ViewBox};

pub enum NodeType {
    Eraser,
    Writer,
}

pub struct Node {
    node_type: NodeType,
    x: u32,
    y: u32,
    last_char: Option<char>,
    white: bool,
    expired: bool,
}

pub struct Column {
    x: u32,
    row_count: u32,
    start_delay: u32,
}

pub struct GreenCodeView {
    content: Vec<Column>,
}




fn main() {
    let mut siv: Cursive = Cursive::default();
    siv.add_fullscreen_layer(Dialog::around(TextView::new("Hello world"))
        .title("cursive")
        .button("quit", |s| s.clear()));
    siv.run();
}
