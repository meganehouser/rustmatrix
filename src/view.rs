use cursive::event::{Event, EventResult};
use cursive::theme::Style;
use cursive::theme::{Color, ColorStyle, Effect};
use cursive::vec::Vec2;
use cursive::view::View;
use cursive::Printer;
use rand::prelude::*;
use std::collections::VecDeque;

const CHARS: &str = "qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNMｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝ1234567890-=*_+|:<>";
const BLANK: char = ' ';
const COLOR_WHITE: u8 = 15;
const COLOR_GREEN: u8 = 2;

#[derive(Clone)]
struct Cell {
    char: char,
    bold: bool,
    white: bool,
}

impl Cell {
    fn new(char: char, bold: bool, white: bool) -> Cell {
        Cell { char, bold, white }
    }

    fn blank() -> Cell {
        Cell {
            char: BLANK,
            bold: false,
            white: false,
        }
    }
}

impl From<&Cell> for Style {
    fn from(cell: &Cell) -> Self {
        let mut style = Self::default();
        if cell.bold {
            style.effects.insert(Effect::Bold);
        };
        if cell.white {
            style.color = Some(ColorStyle::from(Color::from_256colors(COLOR_WHITE)));
        } else {
            style.color = Some(ColorStyle::from(Color::from_256colors(COLOR_GREEN)));
        };
        style
    }
}

enum NodeType {
    Eraser,
    Writer { white: bool },
}

struct InnerNode {
    node_type: NodeType,
    rand: ThreadRng,
}

impl InnerNode {
    fn new(node_type: NodeType, rand: ThreadRng) -> InnerNode {
        InnerNode { node_type, rand }
    }

    fn create_cell(&mut self) -> Cell {
        match self.node_type {
            NodeType::Writer { white: w } => {
                let bold = self.rand.gen();
                let char = self.choice_char();
                Cell::new(char, bold, w.to_owned())
            }
            NodeType::Eraser => Cell::blank(),
        }
    }

    fn choice_char(&mut self) -> char {
        match self.node_type {
            NodeType::Writer { white: _ } => {
                let chars: Vec<char> = String::from(CHARS).chars().collect();
                chars.choose(&mut self.rand).unwrap().to_owned()
            }
            NodeType::Eraser => BLANK,
        }
    }
}

struct Node {
    y: usize,
    inner_node: InnerNode,
}

impl Node {
    fn new(node_type: NodeType) -> Node {
        let y = 0;
        let rand = thread_rng();
        let inner_node = InnerNode::new(node_type, rand);
        Node { y, inner_node }
    }

    fn update(&mut self) {
        self.y = self.y + 1;
    }
}

impl From<&mut Node> for Cell {
    fn from(node: &mut Node) -> Self {
        node.inner_node.create_cell()
    }
}

struct Column {
    row_count: usize,
    wait_time: usize,
    rand: ThreadRng,
    nodes: VecDeque<Node>,
    data: Vec<Cell>,
    is_drawing: bool,
}

impl Column {
    pub fn new(row_count: usize, rand: ThreadRng) -> Column {
        let nodes = VecDeque::new();
        let mut rand_mut = rand;
        let wait_time = rand_mut.gen_range(0, row_count); //rand_mut.gen();

        Column {
            row_count,
            wait_time,
            rand,
            nodes,
            data: vec![Cell::blank(); row_count],
            is_drawing: false,
        }
    }

    fn spawn_node(&mut self) -> Node {
        let max_range = self.row_count - 3;
        let start_delay = self.rand.gen_range(1, max_range);
        self.wait_time = start_delay;

        self.is_drawing = !self.is_drawing;
        if self.is_drawing {
            let white: bool = self.rand.gen();
            Node::new(NodeType::Writer { white })
        } else {
            Node::new(NodeType::Eraser)
        }
    }

    fn update(&mut self) {
        for node in self.nodes.iter_mut() {
            let index = node.y;
            let cell = Cell::from(node);
            if cell.white && index > 0 {
                self.data[index - 1].white = false;
            }
            self.data[index] = cell;
        }

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
            if node.y > self.row_count - 1 {
                self.nodes.pop_front();
            }
        }
    }
}

pub struct GreenCodeView {
    columns: Vec<Column>,
    ticks: u32,
    speed: u32,
}

impl GreenCodeView {
    pub fn new(speed: u32, size: Vec2) -> GreenCodeView {
        let column_count = size.x / 2;
        let columns = (0..column_count)
            .map(|_x| Column::new(size.y, thread_rng()))
            .collect();

        GreenCodeView {
            columns,
            ticks: 0,
            speed,
        }
    }

    fn increment_ticks(&mut self) {
        if self.ticks == self.speed {
            self.ticks = 0;
            self.elapsed()
        } else {
            self.ticks = self.ticks + 1;
        }
    }

    fn elapsed(&mut self) {
        for column in self.columns.iter_mut() {
            column.update();
        }
    }
}

impl View for GreenCodeView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        for (x, column) in self.columns.iter().enumerate() {
            for (y, cell) in column.data.iter().enumerate() {
                let style = Style::from(cell);
                printer.with_style(style, |p| {
                    let s = cell.char.to_owned().to_string();
                    p.print((x * 2, y), &s);
                });
            }
        }
    }
    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if event == Event::Refresh {
            self.increment_ticks();
        }

        EventResult::Consumed(None)
    }
}
