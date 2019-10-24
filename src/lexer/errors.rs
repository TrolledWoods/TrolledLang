use super::{ CodeLocation, SyntaxTreeNode };
use super::super::TreeDump;

pub trait ParserError: CodeLocation + TreeDump {
    fn get_causes(&self) -> &[Box<ParserError>] {
        &[]
    }

    fn get_strength(&self) -> u8;
    fn cmp_strength(&self, other: &Option<Box<ParserError>>) -> bool {
        match other {
            None => true,
            Some(value) => value.get_strength() < self.get_strength()
        }
    }
}

pub struct LiteralError {
    pub start: usize
}

impl LiteralError {
    pub fn new(start: usize) -> LiteralError {
        LiteralError {
            start: start
        }
    }
}

impl CodeLocation for LiteralError {
    fn get_start(&self) -> usize {
        self.start
    }
}

impl TreeDump for LiteralError {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Invalid literal", indent_style.repeat(indent), self.start);
    }
}

impl ParserError for LiteralError {
    fn get_causes(&self) -> &[Box<ParserError>] {
        &[]
    }

    fn get_strength(&self) -> u8 {
        8 // Not sure if this will be kept, a bit too high in my opinion
    }
}

pub enum SimpleError {
    ExpectedBlockOpen(usize),
    ExpectedBlockClose(usize)
}

impl CodeLocation for SimpleError {
    fn get_start(&self) -> usize {
        use SimpleError::*;
        match self {
            ExpectedBlockOpen(loc) => *loc,
            ExpectedBlockClose(loc) => *loc
        }
    }
}

impl TreeDump for SimpleError {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        use SimpleError::*;
        println!("{}({}): {}", indent_style.repeat(indent), self.get_start(), match self {
            ExpectedBlockOpen(_) => "Expected '('",
            ExpectedBlockClose(_) => "Expected ')'"
        });
    }
}

impl ParserError for SimpleError {
    fn get_strength(&self) -> u8 {
        use SimpleError::*;
        match self {
            ExpectedBlockOpen(_) => 0,
            ExpectedBlockClose(_) => 4
        }
    }
}

pub struct BlockError {
    pub start: usize,
    pub strength: u8,
    pub causes: Vec<Box<ParserError>>,
    pub recover: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockError {
    fn get_start(&self) -> usize {
        self.start
    }
}

impl TreeDump for BlockError {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Invalid code block", indent_style.repeat(indent), self.start);
        for cause in self.causes.iter() {
            cause.print_with_indent(indent + 1, indent_style);
        }
    }
}

impl ParserError for BlockError {
    fn get_causes(&self) -> &[Box<ParserError>] {
        self.causes.as_slice()
    }

    fn get_strength(&self) -> u8 {
        self.strength
    }
}

pub type ParseResult<T> = Result<T, Box<ParserError>>;
