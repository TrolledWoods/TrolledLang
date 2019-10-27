use super::{ CodeLocation, SyntaxTreeNode };
use super::super::TreeDump;
use super::super::needle::Loc;

pub trait ParserError: CodeLocation + TreeDump {
    fn get_causes(&self) -> &[Box<ParserError>] {
        &[]
    }

    fn get_strength(&self) -> u8;
    fn cmp_strength(&self, other: &Option<Box<ParserError>>) -> bool {
        match other {
            None => true,
            Some(value) => value.get_strength() <= self.get_strength()
        }
    }
}

pub struct LiteralError {
    pub start: Loc
}

impl LiteralError {
    pub fn new(start: Loc) -> LiteralError {
        LiteralError {
            start: start
        }
    }
}

impl CodeLocation for LiteralError {
    fn get_start(&self) -> Loc {
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
        1
    }
}

pub enum SimpleError {
    ExpectedBlockOpen(Loc),
    ExpectedBlockClose(Loc),
    ExpectedIdentifier(Loc, u8),
    ExpectedEquals(Loc, u8),
    InvalidVariableName(Loc, u8),
    ExpectedExpression(Loc, u8)
}

impl CodeLocation for SimpleError {
    fn get_start(&self) -> Loc {
        use SimpleError::*;
        match self {
            ExpectedBlockOpen(loc) => *loc,
            ExpectedBlockClose(loc) => *loc,
            ExpectedIdentifier(loc, _) => *loc,
            ExpectedEquals(loc, _) => *loc,
            InvalidVariableName(loc, _) => *loc,
            ExpectedExpression(loc, _) => *loc,
        }
    }
}

impl TreeDump for SimpleError {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        use SimpleError::*;
        println!("{}({}): {}", indent_style.repeat(indent), self.get_start(), match self {
            ExpectedBlockOpen(_) => "Expected '('",
            ExpectedBlockClose(_) => "Expected ')'",
            ExpectedIdentifier(_, _) => "Expected identifier",
            ExpectedEquals(_, _) => "Expected equals",
            InvalidVariableName(_, _) => "Invalid variable name",
            ExpectedExpression(_, _) => "Expected expression",
        });
    }
}

impl ParserError for SimpleError {
    fn get_strength(&self) -> u8 {
        use SimpleError::*;
        match self {
            ExpectedBlockOpen(_) => 0,
            ExpectedBlockClose(_) => 4,
            ExpectedIdentifier(_, strength) => *strength,
            ExpectedEquals(_, strength) => *strength,
            InvalidVariableName(_, strength) => *strength,
            ExpectedExpression(_, strength) => *strength,
        }
    }
}

pub struct AssignmentDataError {
    pub start: Loc,
    pub strength: u8,
    pub cause: Box<ParserError>,
    pub var_name: String
}

impl CodeLocation for AssignmentDataError {
    fn get_start(&self) -> Loc {
        self.start
    }
}

impl TreeDump for AssignmentDataError {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Invalid assignment for '{}'", indent_style.repeat(indent), self.start, self.var_name);
        self.cause.print_with_indent(indent + 1, indent_style);
    }
}

impl ParserError for AssignmentDataError {
    fn get_causes(&self) -> &[Box<ParserError>] {
        &[] // Bah
    }

    fn get_strength(&self) -> u8 {
        self.strength
    }
}

pub struct BlockError {
    pub start: Loc,
    pub strength: u8,
    pub causes: Vec<Box<ParserError>>,
    pub recover: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockError {
    fn get_start(&self) -> Loc {
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
