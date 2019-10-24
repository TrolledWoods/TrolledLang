use super::{ CodeLocation, SyntaxTreeNode };

pub trait ParserError: CodeLocation {
    fn print(&self) {
        self.print_indent(&String::from(" | "), 0);
    }
    fn print_indent(&self, indent: &String, n_indent: usize);
    
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

impl ParserError for LiteralError {
    fn print_indent(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Invalid literal", indent.repeat(n_indent), self.start);
    }

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

impl ParserError for SimpleError {
    fn print_indent(&self, indent: &String, n_indent: usize) {
        use SimpleError::*;
        println!("{}({}): {}", indent.repeat(n_indent), self.get_start(), match self {
            ExpectedBlockOpen(_) => "Expected '('",
            ExpectedBlockClose(_) => "Expected ')'"
        });
    }

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

impl ParserError for BlockError {
    fn print_indent(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Invalid code block", indent.repeat(n_indent), self.start);
        for cause in self.causes.iter() {
            cause.print_indent(indent, n_indent + 1);
        }
    }

    fn get_causes(&self) -> &[Box<ParserError>] {
        self.causes.as_slice()
    }

    fn get_strength(&self) -> u8 {
        self.strength
    }
}

pub type ParseResult<T> = Result<T, Box<ParserError>>;
