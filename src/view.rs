use rand::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{stdout, Stdout, Write};
use std::mem;
use termion;
use termion::raw::{IntoRawMode, RawTerminal};

const CHARS: &str = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNMｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890-=*_+|:<>";

enum ColorType {
    White,
    Normal,
}

enum Character {
    Char {
        char: char,
        bold: bool,
        color_type: ColorType,
    },
    Blank,
}

enum NodeType {
    Eraser,
    Writer { white: bool, rng: ThreadRng },
}

impl NodeType {
    fn choice_char(&mut self) -> Character {
        match self {
            NodeType::Writer { white, ref mut rng } => {
                let chars: Vec<char> = String::from(CHARS).chars().collect();
                let char = chars.choose(rng).unwrap().to_owned();
                let bold = rng.gen();
                let color_type = if *white {
                    ColorType::White
                } else {
                    ColorType::Normal
                };
                Character::Char {
                    char,
                    bold,
                    color_type,
                }
            }
            NodeType::Eraser => Character::Blank,
        }
    }
}

struct Node {
    node_type: NodeType,
    y: u16,
    previous_char: Character,
    char: Character,
}

impl Node {
    fn new(mut node_type: NodeType) -> Node {
        let y = 1;
        let char = node_type.choice_char();
        Node {
            node_type,
            y,
            previous_char: Character::Blank,
            char,
        }
    }

    fn update(&mut self) {
        self.y += 1;
        let next_char = self.node_type.choice_char();
        self.previous_char = mem::replace(&mut self.char, next_char);
    }
}

struct Column {
    row_count: u16,
    wait_time: u16,
    rng: ThreadRng,
    nodes: VecDeque<Node>,
    is_drawing: bool,
}

impl Column {
    fn new(row_count: u16) -> Column {
        let mut rng = thread_rng();
        let wait_time = rng.gen_range(0, row_count);
        Column {
            row_count,
            wait_time,
            rng,
            nodes: VecDeque::new(),
            is_drawing: false,
        }
    }

    fn spawn_node(&mut self) -> Node {
        let max_range = self.row_count - 3;
        let start_delay = self.rng.gen_range(1, max_range);
        self.wait_time = start_delay;

        self.is_drawing = !self.is_drawing;
        if self.is_drawing {
            let white: bool = self.rng.gen();
            Node::new(NodeType::Writer {
                white,
                rng: thread_rng(),
            })
        } else {
            Node::new(NodeType::Eraser)
        }
    }

    fn update(&mut self) {
        for node in self.nodes.iter_mut() {
            node.update();
        }

        if self.wait_time == 0 {
            let node = self.spawn_node();
            self.nodes.push_back(node);
        } else {
            self.wait_time -= 1;
        }

        if let Some(node) = self.nodes.front() {
            if node.y > self.row_count {
                self.nodes.pop_front();
            }
        }
    }
}

pub struct MatrixApp {
    columns: Vec<Column>,
    stdout: RefCell<RawTerminal<Stdout>>,
}

impl MatrixApp {
    pub fn new() -> MatrixApp {
        let (size_x, size_y) = termion::terminal_size().unwrap();
        let mut stdout = stdout().into_raw_mode().unwrap();
        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();
        let column_count = size_x / 2;

        let columns = (0..column_count).map(|_| Column::new(size_y)).collect();

        MatrixApp {
            columns,
            stdout: RefCell::new(stdout),
        }
    }

    fn update(&mut self) {
        for column in self.columns.iter_mut() {
            column.update();
        }
    }

    fn draw(&self) {
        for (x, column) in self.columns.iter().enumerate() {
            for node in column.nodes.iter() {
                write!(
                    self.stdout.borrow_mut(),
                    "{}",
                    termion::cursor::Goto((x * 2) as u16, node.y)
                )
                .unwrap();

                match &node.char {
                    Character::Char {
                        char,
                        bold,
                        color_type,
                    } => {
                        match color_type {
                            ColorType::White => {
                                self.set_white_char_style();
                            }
                            ColorType::Normal => {
                                self.set_normal_char_style(*bold);
                            }
                        };
                        write!(
                            self.stdout.borrow_mut(),
                            "{}{}",
                            char,
                            termion::style::Reset
                        )
                        .unwrap();
                    }
                    Character::Blank => {
                        write!(self.stdout.borrow_mut(), " ").unwrap();
                    }
                }

                if node.y == 1 {
                    continue;
                }

                if let Character::Char {
                    char,
                    bold,
                    color_type: ColorType::White,
                } = &node.char
                {
                    self.set_normal_char_style(*bold);
                    write!(
                        self.stdout.borrow_mut(),
                        "{}{}{}",
                        termion::cursor::Goto((x * 2) as u16, (node.y - 1) as u16),
                        char,
                        termion::style::Reset
                    )
                    .unwrap();
                }
            }
        }
        self.stdout.borrow_mut().flush().unwrap();
    }

    fn set_normal_char_style(&self, bold: bool) {
        if bold {
            write!(self.stdout.borrow_mut(), "{}", termion::style::Bold,).unwrap();
        }

        write!(
            self.stdout.borrow_mut(),
            "{}",
            termion::color::Fg(termion::color::Green)
        )
        .unwrap();
    }

    fn set_white_char_style(&self) {
        write!(
            self.stdout.borrow_mut(),
            "{}{}",
            termion::style::Bold,
            termion::color::Fg(termion::color::White)
        )
        .unwrap();
    }

    pub fn on_tick(&mut self) {
        self.update();
        self.draw();
    }
}

impl Drop for MatrixApp {
    fn drop(&mut self) {
        write!(self.stdout.borrow_mut(), "{}", termion::cursor::Show).unwrap();
    }
}
