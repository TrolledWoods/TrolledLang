use super::tokenizer::{ Token };
use super::tokenizer;
use super::needle::{ Needle, Loc, TextMetaData };
use super::TreeDump;

mod errors;
use errors::{ BlockError, LiteralError, ParseResult };
use errors::SimpleError::*;

pub trait CodeLocation {
    fn get_start(&self) -> Loc;
}

pub trait SyntaxTreeNode: CodeLocation + TreeDump {
}

pub struct ErrorNode {
    start: Loc
}

impl CodeLocation for ErrorNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for ErrorNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Error", indent_style.repeat(indent), self.start);
    }
}

impl SyntaxTreeNode for ErrorNode {}

pub struct LiteralNode {
    start: Loc,
    literal: tokenizer::LiteralType
}

impl CodeLocation for LiteralNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for LiteralNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): literal {}", indent_style.repeat(indent), self.start, self.literal);
    }   
}

impl SyntaxTreeNode for LiteralNode {}

pub struct NilNode {
    start: Loc
}

impl CodeLocation for NilNode {
    fn get_start(&self) -> Loc { self.start }
}

impl TreeDump for NilNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Nil", indent_style.repeat(indent), self.start);
    }
}

impl SyntaxTreeNode for NilNode {}

pub struct BlockNode {
    start: Loc,
    contents: Vec<Box<SyntaxTreeNode>>,
    _return: Option<Box<SyntaxTreeNode>>
}

impl CodeLocation for BlockNode {
    fn get_start(&self) -> Loc { self.start }
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


fn parse_literal(tokens: &mut Needle<Token>, meta: &TextMetaData) -> ParseResult<Box<SyntaxTreeNode>> {
    if let Some(token) = tokens.read() {
        if let Some(literal) = token.as_literal() {
            Ok(Box::new(LiteralNode { start: token.start, literal: literal }))
        }else {
            Err(Box::new(LiteralError::new(token.start)))
        }
    }else{
        Err(Box::new(LiteralError::new(meta.get_end())))
    }
}

pub fn parse_value(tokens: &mut Needle<Token>, meta: &TextMetaData) -> ParseResult<Box<SyntaxTreeNode>> {
    let mut current_error = None;
    let mut current_error_end = 0;
    
    tokens.push_state();
    let result = parse_block(tokens, meta);
    match result {
        Ok(value) => {
            tokens.pop_state_no_revert();
            return Ok(value);
        },
        Err(error) => {
            if error.cmp_strength(&current_error) {
                current_error_end = tokens.get_index();
                current_error = Some(error);
            }
        }
    }
    tokens.pop_state();

    tokens.push_state();
    let result = parse_literal(tokens, meta);
    match result {
        Ok(value) => {
            tokens.pop_state_no_revert();
            return Ok(value);
        },
        Err(error) => {
            if error.cmp_strength(&current_error) {
                current_error_end = tokens.get_index();
                current_error = Some(error);
            }
        }
    }
    tokens.pop_state();

    if let Some(error) = current_error {
        tokens.index = current_error_end;
        return Err(error);
    }else {
        panic!("No error was given, I have no idea why");
    }
} 

pub fn parse_block(tokens: &mut Needle<Token>, meta: &TextMetaData) 
        -> ParseResult<Box<SyntaxTreeNode>> {
    use tokenizer::KeywordType;

    let start = match tokens.peek() {
        Some(t) => t.start,
        None => Loc::new(500, 500)
    };

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
            start: meta.get_end(),
            strength: 1, 
            causes: vec![Box::new(ExpectedBlockOpen(start))],
            recover: None
        }));
    }
    if next.unwrap().is_keyword(KeywordType::BlockClose) {
        tokens.next();
        return Ok(Box::new(NilNode { start: start }));
    }

    let mut contents: Vec<Box<SyntaxTreeNode>> = Vec::new();
    let mut _return = None;
    let mut errors = Vec::new();

    loop {
        let current_value = match parse_value(tokens, meta) {
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
                errors.push(Box::new(ExpectedBlockClose(current_value.get_start())));
                return Err(Box::new(BlockError {
                    start: start,
                    causes: errors,
                    strength: 2,
                    recover: Some(Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    }))
                }));
            }
        }else{
            errors.push(Box::new(ExpectedBlockClose(meta.get_end())));
            return Err(Box::new(BlockError {
                    start: start,
                    strength: 2,
                    causes: errors,
                    recover: Some(Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    }))
                }));
        }
    }

    if errors.len() > 0 {
        Err(
            Box::new(BlockError {
                start: start,
                strength: 4,
                causes: errors,
                recover: Some(
                    Box::new(BlockNode {
                        start: start,
                        contents: contents,
                        _return: _return
                    })
                )
            })
        )
    }else{
        Ok(Box::new(BlockNode {
            start: start,
            contents: contents,
            _return: _return
        }))
    }
}