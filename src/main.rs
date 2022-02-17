use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::element::{Group, Line, Text};
use svg::{Document, Node};

fn print_char(c: u32) {
    let mut cc = c << 8;
    for i in 0..32 {
        if cc & 0x80_00_00_00 == 0x80_00_00_00 {
            print!("*");
        } else {
            print!(" ");
        }
        cc = cc << 1;
        if i % 4 == 3 {
            println!();
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct PixChar4x6(u32);

impl PixChar4x6 {
    pub fn new(c: char) -> PixChar4x6 {
        PixChar4x6(match c {
            '0' => 0b0110_1001_1011_1101_1001_0110u32,
            '1' => 0b0100_1100_0100_0100_0100_1110u32,
            '2' => 0b0110_1001_0001_0110_1000_1111u32,
            '3' => 0b0110_0001_0010_0001_1001_0110u32,
            '4' => 0b0010_0110_1010_1111_0010_0010u32,
            '5' => 0b1111_1000_1110_0001_1001_0110u32,
            '6' => 0b0110_1000_1110_1010_1001_0110u32,
            '7' => 0b1111_0001_0010_0100_0100_0100u32,
            '8' => 0b0110_1001_0110_1001_1001_0110u32,
            '9' => 0b0110_1001_1001_0111_0001_0110u32,
            '.' => 0b0000_0000_0000_0000_0000_0100u32,
            _ => 0,
        })
    }

    /// - `row`: 0..6
    pub fn row(self, row: usize) -> u32 {
        assert!(row < 6, "row {} must be in the range 0..6", row);
        (self.0 >> (5 - row) * 4) & 0x0fu32
    }

    /// - `row`: 0..6
    /// - `col`: 0..4
    pub fn pix(self, row: usize, col: usize) -> bool {
        assert!(row < 6, "row {} must be in the range 0..6", row);
        assert!(col < 4, "row {} must be in the range 0..6", row);
        let mask = 1 << (3 - col);
        self.row(row) & mask == mask
    }

    pub fn print_bin(self) {
        for i in 0..6 {
            println!("{:04b}", self.row(i))
        }
    }

    pub fn print(self) {
        for row in 0..6 {
            let mut l = String::new();
            for col in 0..4 {
                if self.pix(row, col) {
                    l.push('*');
                } else {
                    l.push(' ');
                }
            }
            println!("{}", l);
        }
    }
}

struct Board([u32; 32]);

impl Board {
    pub fn new() -> Board {
        Board([0u32; 32])
    }

    pub fn put_char(&mut self, c: char, x: usize, y: usize) {
        let c = PixChar4x6::new(c);
        for r in 0..6 {
            self.0[r + y] = self.0[r + y] | (c.row(r) << (32 - x - 4))
        }
    }

    pub fn put_string(&mut self, s: &str, x: usize, y: usize) {
        let mut x_char = x;
        let mut y_char = y;
        for c in s.chars() {
            if c == '\n' {
                x_char = x;
                y_char = y_char + 7;
            } else {
                self.put_char(c, x_char, y_char);
                x_char = x_char + 5;
            }
        }
    }

    /// `x`: 0..32
    /// `y`: 0..32
    pub fn print(&self, x: usize, y: usize) {
        for i in 0..y {
            println!("{:032b}", self.0[i]);
        }
    }

    pub fn to_9(&self) {
        let (max_x, max_y) = self.find_max_used();
        let mut document = Document::new()
            .set("width", format!("{}px", (max_x * 30 + 1) * 2))
            .set("height", format!("{}px", (max_y * 30 + 1) * 2))
            .set("viewBox", (0, 0, max_x * 30 + 1, max_y * 30 + 1));
        let mut values = Group::new()
            .set("font-family", "Verdana")
            .set("font-size", 24)
            .set("text-anchor", "middle");
        for y in 0..max_y {
            for x in 0..max_x {
                let v = (self.0[y] & (0xe000_0000 >> x)).count_ones()
                    + (self.0[y + 1] & (0xe000_0000 >> x)).count_ones()
                    + (self.0[y + 2] & (0xe000_0000 >> x)).count_ones();
                print!("{}", v);
                values = values.add(
                    Text::new()
                        .set("x", x * 30 + 15)
                        .set("y", y * 30 + 25)
                        .add(svg::node::Text::new(v.to_string())),
                );
            }
            println!();
        }
        document = document.add(values);

        let mut lines = Group::new().set("stroke-width", 1).set("stroke", "blue");
        for i in 0..(max_y + 1) {
            lines = lines.add(
                Line::new()
                    .set("x1", 0)
                    .set("y1", i * 30)
                    .set("x2", max_x * 30 + 1)
                    .set("y2", i * 30),
            );
        }
        for i in 0..(max_x + 1) {
            lines = lines.add(
                Line::new()
                    .set("x1", i * 30)
                    .set("y1", 0)
                    .set("x2", i * 30)
                    .set("y2", max_y * 30 + 1),
            );
        }
        document = document.add(lines);
        svg::save("image.svg", &document).unwrap();
    }

    fn find_max_used(&self) -> (usize, usize) {
        let x = 32 - self.0.iter().fold(0, |a, b| a | b).trailing_zeros() as usize;
        let y = self
            .0
            .iter()
            .enumerate()
            .fold(0, |a, (b, &c)| if c == 0 { a } else { b })
            + 1;
        (x, y)
    }
}

fn main() {
    let mut b = Board::new();
    b.put_string("1234\n5678", 1, 1);
    // b.put_string("5678", 1, 8);
    b.print(32, 16);
    println!("{:?}", b.find_max_used());
    b.to_9();
}
