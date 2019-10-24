use super::tokenizer::{ Token };
use super::tokenizer;
use super::Needle;
use super::TreeDump;

mod errors;
use errors::{ BlockError, LiteralError, ParseResult };
use errors::SimpleError::*;

pub trait CodeLocation {
    fn get_start(&self) -> usize;
}

pub trait SyntaxTreeNode: CodeLocation + TreeDump {
}

pub struct ErrorNode {
    start: usize
}

impl CodeLocation for ErrorNode {
    fn get_start(&self) -> usize { self.start }
}

impl TreeDump for ErrorNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Error", indent_style.repeat(indent), self.start);
    }
}

impl SyntaxTreeNode for ErrorNode {}

pub struct LiteralNode {
    start: usize,
    literal: tokenizer::LiteralType
}

impl CodeLocation for LiteralNode {
    fn get_start(&self) -> usize { self.start }
}

impl TreeDump for LiteralNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Literal TODO: Add literal name here", indent_style.repeat(indent), self.start);
    }   
}

impl SyntaxTreeNode for LiteralNode {}

pub struct NilNode {
    start: usize
}

impl CodeLocation for NilNode {
    fn get_start(&self) -> usize { self.start }
}

impl TreeDump for NilNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Nil", indent_style.repeat(indent), self.start);
    }
}

impl SyntaxTreeNode for NilNode {}

pub struct BlockNode {
    start: usize,
    contents: Vec<Box<SyntaxTreeNode>>,
    _return: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockNode {
    fn get_start(&self) -> usize { self.start }
}

impl TreeDump for BlockNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Block", indent_style.repeat(indent), self.start);
        if self.contents.len() > 0 {
            println!("{}Contents:", indent_style.repeat(indent + 1));
            for content in &self.contents {
                content.print_with_indent(indent + 2, indent_style);
            }
        }
        if let Some(node) = &self._return {
            println!("{}Returns:", indent_style.repeat(indent + 1));
            node.print_with_indent(indent + 2, indent_style);
        }
    }
}

impl SyntaxTreeNode for BlockNode {}


fn parse_literal(tokens: &mut Needle<Token>) -> ParseResult<Box<SyntaxTreeNode>> {
    if let Some(token) = tokens.read() {
        if let Some(literal) = token.as_literal() {
            Ok(Box::new(LiteralNode { start: token.start, literal: literal }))
        }else {
            Err(Box::new(LiteralError::new(9999)))
        }
    }else{
        Err(Box::new(LiteralError::new(9999)))
    }
}

fn parse_value(tokens: &mut Needle<Token>) -> ParseResult<Box<SyntaxTreeNode>> {
    let mut current_error = None;
    
    tokens.push_state();
    let result = parse_literal(tokens);
    match result {
        Ok(value) => {
            tokens.pop_state_no_revert();
            return Ok(value);
        },
        Err(error) => {
            if error.cmp_strength(&current_error) {
                current_error = Some(error);
            }
        }
    }
    tokens.pop_state();

    if let Some(error) = current_error {
        return Err(error);
    }else {
        panic!("No error was given, I have no idea why");
    }
} 

pub fn parse_block(tokens: &mut Needle<Token>) -> ParseResult<Box<SyntaxTreeNode>> {
    use tokenizer::KeywordType;

    let start = tokens.get_index();

    if !tokens.match_func_offset(0, | t | t.is_keyword(KeywordType::BlockOpen)) {
        return Err(Box::new(BlockError {
            start: start,
            strength: 0,
            causes: vec![Box::new(ExpectedBlockOpen(start))],
            recover: None
        }));
    }

    tokens.next();
    let next = tokens.peek();
    if next.is_none() { 
        return Err(Box::new(BlockError {
            start: 9999999,
            strength: 1, 
            causes: vec![Box::new(ExpectedBlockOpen(start))],
            recover: None
        }));
    }
    if next.unwrap().is_keyword(KeywordType::BlockClose) {
        return Ok(Box::new(NilNode { start: start }));
    }

    let mut contents: Vec<Box<SyntaxTreeNode>> = Vec::new();
    let mut _return = None;
    let mut errors = Vec::new();

    loop {
        let current_value = match parse_value(tokens) {
            Ok(value) => value,
            Err(err) => {
                let start = err.get_start();
                errors.push(err);
                Box::new(ErrorNode { start: start })
            }
        };

        if let Some(token) = tokens.read() {
            if token.is_keyword(KeywordType::BlockClose) {
                _return = Some(current_value);
                break;
            } else if token.is_keyword(KeywordType::BlockSeparator) {
                contents.push(current_value);

                if let Some(token) = tokens.peek() {
                    if token.is_keyword(KeywordType::BlockClose) {
                        break;
                    }
                }
            }else {
                return Err(Box::new(BlockError {
                    start: start,
                    causes: vec![Box::new(ExpectedBlockClose(current_value.get_start()))],
                    strength: 2,
                    recover: Some(Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    }))
                }));
            }
        }else{
            return Err(Box::new(BlockError {
                    start: start,
                    strength: 2,
                    causes: vec![Box::new(ExpectedBlockClose(current_value.get_start()))],
                    recover: Some(Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    }))
                }));
        }
    }

    Ok(Box::new(BlockNode {
        start: start,
        contents: contents,
        _return: _return
    }))
}
