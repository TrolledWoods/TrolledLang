use super::tokenizer::{ Token };
use super::tokenizer;
pub use super::needle::{ Needle, Loc, TextMetaData };
use super::TreeDump;
use std::collections::HashMap;

mod type_handler;
mod errors;
pub use type_handler::{ Type, TypeCollection, ScopePool, ScopeHandle };
pub use errors::{ BlockError, LiteralError, AssignmentDataError };
use errors::SimpleError::*;
pub use errors::SimpleError;
pub use errors::ParseResult;

mod block_node;
mod literal_node;
mod assignment_node;
use block_node::BlockNode;
use assignment_node::AssignmentNode;
use literal_node::LiteralNode;

pub trait CodeLocation {
    fn get_start(&self) -> Loc;
}

pub trait SyntaxTreeNode: CodeLocation + TreeDump {
    fn get_possible_returns(&self, scope: ScopeHandle, scopes: &ScopePool) -> TypeCollection {
        TypeCollection::from(Vec::new())
    }
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

pub struct VariableNode {
    start: Loc,
    identifier: String
}

impl VariableNode {
    fn new(start: Loc, identifier: String) -> VariableNode {
        VariableNode {
            start: start,
            identifier: identifier
        }
    }
}

impl TreeDump for VariableNode {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        println!("{}({}): Variable '{}'", indent_style.repeat(indent), self.start, self.identifier);
    }
}

impl CodeLocation for VariableNode {
    fn get_start(&self) -> Loc { self.start }
}

impl SyntaxTreeNode for VariableNode {
    fn get_possible_returns(&self, scope: ScopeHandle, scopes: &ScopePool) -> TypeCollection {
        scope.get(scopes, &self.identifier[..])
            .expect("A VariableNode's variable name does not fit the scope")
            .clone()
    }
}

fn parse_variable(tokens: &mut Needle<Token>, meta: &TextMetaData, scope: ScopeHandle, scopes: &ScopePool)
        -> ParseResult<Box<SyntaxTreeNode>> {
    let next = match tokens.read() {
        Some(token) => token,
        None => return Err(Box::new(SimpleError::ExpectedIdentifier(meta.get_end(), 0)))
    };

    if let tokenizer::TokenType::Identifier(string) = &next.token_type {
        if scope.get(scopes, &string[..]).is_some() {
            Ok(Box::new(VariableNode::new(next.start, string.clone())))
        }else{
            Err(Box::new(SimpleError::InvalidVariableName(next.start, 1)))
        }
    }else{
        Err(Box::new(SimpleError::ExpectedIdentifier(next.start, 0)))
    }
}

fn parse_assignment(tokens: &mut Needle<Token>, meta: &TextMetaData, scope: ScopeHandle, scopes: &mut ScopePool) 
        -> ParseResult<Box<SyntaxTreeNode>> {
    // Identifier
    let next = match tokens.read() {
        Some(token) => token,
        None => return Err(Box::new(SimpleError::ExpectedIdentifier(meta.get_end(), 0)))
    };

    let start = next.start;
    let identifier = match &next.token_type {
        tokenizer::TokenType::Identifier(name) => name,
        _ => return Err(Box::new(SimpleError::ExpectedIdentifier(next.start, 0)))
    }.clone();

    // Equals
    let next = match tokens.read() {
        Some(token) => token,
        None => return Err(Box::new(SimpleError::ExpectedEquals(meta.get_end(), 0)))
    };
    if !next.is_keyword(tokenizer::KeywordType::Assign) {
        return Err(Box::new(SimpleError::ExpectedEquals(next.start, 0)));
    }

    // Assign to
    let data = parse_value(tokens, meta, scope, scopes);
    match data {
        Ok(data) => {
            let possible_returns = data.get_possible_returns(scope, scopes);
            if possible_returns.is_undef() {
                return Err(Box::new(AssignmentDataError {
                    start: start,
                    strength: 3,
                    cause: Box::new(SimpleError::ExpectedExpression(data.get_start(), 3)),
                    var_name: identifier
                }));
            }

            if scope.get(scopes, &identifier[..]).is_none() {
                scope.insert(scopes, &identifier[..], possible_returns);
            }else{
                let element = scope.get_mut(scopes, &identifier[..]).unwrap();
                element.constrain(&possible_returns);
            }
            Ok(Box::new(AssignmentNode {
                start: start,
                identifier: identifier,
                data: data
            }))
        },
        Err(error) => Err(Box::new(AssignmentDataError {
            start: start,
            strength: 3,
            cause: error,
            var_name: identifier
        }))
    } 

    
    
    // Identifier
    // let next = tokens.read().map(|token| token.as_identifier(0));
    // if next.is_none() {
    //     return Err(Box::new(SimpleError::expected_identifier(meta.get_end(), 0)));
    // }
    // let identifier = next.unwrap();
    
    // Equals
    
}

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

pub fn parse_value(tokens: &mut Needle<Token>, meta: &TextMetaData, scope: ScopeHandle, scopes: &mut ScopePool) 
        -> ParseResult<Box<SyntaxTreeNode>> {
    let mut current_error = None;
    let mut current_error_end = 0;
    
    tokens.push_state();
    let result = parse_block(tokens, meta, scope, scopes);
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
    let result = parse_assignment(tokens, meta, scope, scopes);
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
    
    tokens.push_state();
    let result = parse_variable(tokens, meta, scope, scopes);
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

pub fn parse_block(tokens: &mut Needle<Token>, meta: &TextMetaData, parent_scope: ScopeHandle, scopes: &mut ScopePool) 
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
    let scope = parent_scope.create_subscope(scopes);

    loop {
        let current_value = match parse_value(tokens, meta, scope, scopes) {
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
                        tokens.next();
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
                        scope: scope,
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
                        scope: scope,
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
                        scope: scope,
                        contents: contents,
                        _return: _return
                    })
                )
            })
        )
    }else{
        // If you return something, it cannot be of type undef
        if let Some(r) = &_return {
            if r.get_possible_returns(scope, scopes).is_undef() {
                errors.push(Box::new(SimpleError::ExpectedExpression(r.get_start(), 4)));
                contents.push(_return.unwrap());
                return Err(
                    Box::new(BlockError {
                        start: start,
                        strength: 4,
                        causes: errors,
                        recover: Some(
                            Box::new(BlockNode {
                                start: start,
                                scope: scope,
                                contents: contents,
                                _return: None
                            })
                        )
                    })
                );
            }
        }

        Ok(Box::new(BlockNode {
            start: start,
            scope: scope,
            contents: contents,
            _return: _return
        }))
    }
}