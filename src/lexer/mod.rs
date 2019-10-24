use super::tokenizer::{ Token };
use super::tokenizer;
use super::Needle;

mod errors;
use errors::{ BlockError, LiteralError, ParseResult };
use errors::SimpleError::*;

pub trait CodeLocation {
    fn get_start(&self) -> usize;
}

pub trait SyntaxTreeNode: CodeLocation {
    fn print(&self, indent: &String, indent: usize);
}

pub struct ErrorNode {
    start: usize
}

impl CodeLocation for ErrorNode {
    fn get_start(&self) -> usize { self.start }
}

impl SyntaxTreeNode for ErrorNode {
    fn print(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Error", indent.repeat(n_indent), self.start);
    }
}

pub struct LiteralNode {
    start: usize,
    literal: tokenizer::Literal
}

impl CodeLocation for LiteralNode {
    fn get_start(&self) -> usize { self.start }
}

impl SyntaxTreeNode for LiteralNode {
    fn print(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Literal TODO: Add literal name here", indent.repeat(n_indent), self.start);
    }   
}

pub struct NilNode {
    start: usize
}

impl CodeLocation for NilNode {
    fn get_start(&self) -> usize { self.start }
}

impl SyntaxTreeNode for NilNode {
    fn print(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Nil", indent.repeat(n_indent), self.start);
    }
}

pub struct BlockNode {
    start: usize,
    contents: Vec<Box<SyntaxTreeNode>>,
    _return: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockNode {
    fn get_start(&self) -> usize { self.start }
}

impl SyntaxTreeNode for BlockNode {
    fn print(&self, indent: &String, n_indent: usize) {
        println!("{}({}): Block", indent.repeat(n_indent), self.start);
        if self.contents.len() > 0 {
            println!("{}Contents:", indent.repeat(n_indent + 1));
            for content in &self.contents {
                content.print(indent, n_indent + 2);
            }
        }
        if let Some(node) = &self._return {
            println!("{}Returns:", indent.repeat(n_indent + 1));
            node.print(indent, n_indent + 2);
        }
    }
}


fn parse_literal(tokens: &mut Needle<Token>) -> ParseResult<Box<SyntaxTreeNode>> {
    let thing = tokens.read();
    match thing {
        Some(Token::_Literal(start, literal)) => {
            Ok(
                Box::from(
                    LiteralNode {
                        start: *start,
                        literal: literal.clone()
                    }
                )
            )
        },
        _ => {
            Err(Box::from(LiteralError::new(9999)))
        }
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
    let start = tokens.get_index();

    if !tokens.match_func_offset(0, | t | t.is_keyword(tokenizer::Keyword::BlockOpen)) {
        return Err(Box::new(BlockError {
            start: start,
            strength: 0,
            causes: vec![Box::from(ExpectedBlockOpen(start))],
            recover: None
        }));
    }

    tokens.next();
    let next = tokens.peek();
    if next.is_none() { 
        return Err(Box::new(BlockError {
            start: 9999999,
            strength: 1, 
            causes: vec![Box::from(ExpectedBlockOpen(start))],
            recover: None
        }));
    }
    if next.unwrap().is_keyword(tokenizer::Keyword::BlockClose) {
        return Ok(Box::new(NilNode { start: start }));
    }

    let mut contents = Vec::new();
    let mut _return = None;
    let mut errors = Vec::new();

    loop {
        let current_value = parse_value(tokens);
        let current_value = match current_value {
            Ok(value) => value,
            Err(err) => {
                let start = err.get_start();
                errors.push(err);
                Box::from(ErrorNode { start: start })
            }
        };

        if let Some(token) = tokens.read() {
            if token.is_keyword(tokenizer::Keyword::BlockClose) {
                _return = Some(current_value);
                break;
            } else if token.is_keyword(tokenizer::Keyword::BlockSeparator) {
                contents.push(current_value);

                if let Some(token) = tokens.peek() {
                    if token.is_keyword(tokenizer::Keyword::BlockClose) {
                        break;
                    }
                }
            }else {
                return Err(Box::new(BlockError {
                    start: start,
                    causes: vec![Box::from(ExpectedBlockClose(current_value.get_start()))],
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
                    causes: vec![Box::from(ExpectedBlockClose(current_value.get_start()))],
                    recover: Some(Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    }))
                }));
        }
    }

    Ok(Box::from(BlockNode {
        start: start,
        contents: contents,
        _return: _return
    }))
}
