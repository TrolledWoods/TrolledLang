use super::Needle;
use std::vec::Vec;
use super::TreeDump;

pub struct Error {
    pub msg: &'static str,
    pub loc: usize,
    pub priority: u8
}

impl Error {
    pub fn new(loc: usize, priority: u8, msg: &'static str) -> Error {
        Error {
            loc: loc,
            msg: msg,
            priority: priority
        }
    }

    pub fn at_needle<T>(needle: &Needle<T>, priority: u8, msg: &'static str) -> Error {
        Error {
            loc: needle.get_index(),
            msg: msg,
            priority: priority
        }
    }
}

#[derive(Clone)]
pub enum TokenType {
    Literal(LiteralType),
    Operator(OperatorType),
    Keyword(KeywordType),
    Identifier(String)
}

// impl Clone for TokenType {
//     fn clone(&self) -> TokenType {
//         use TokenType::*;
//         match self {
//             Literal(literal) => Literal(literal.clone()),
//             Operator(operator) => Operator(operator.clone()),
//             Keyword(keyword) => Keyword(keyword.clone()),
//             Identifier(string) => Identifier(string.clone())
//         }
//     }
// }

#[derive(Clone)]
pub struct Token {
    pub start: usize,
    pub token_type: TokenType // Can't use 'type' as var. name cuz that's a keyword :(
}

impl Token {
    pub fn literal(start: usize, literal: LiteralType) -> Token {
        Token {
            start: start,
            token_type: TokenType::Literal(literal)
        }
    }

    pub fn operator(start: usize, operator: OperatorType) -> Token {
        Token {
            start: start,
            token_type: TokenType::Operator(operator)
        }
    }

    pub fn keyword(start: usize, keyword: KeywordType) -> Token {
        Token {
            start: start,
            token_type: TokenType::Keyword(keyword)
        }
    }
    
    pub fn identifier(start: usize, identifier: String) -> Token {
        Token {
            start: start,
            token_type: TokenType::Identifier(identifier)
        }
    }

    pub fn as_literal(&self) -> Option<LiteralType> {
        if let TokenType::Literal(literal) = &self.token_type {
            Some(literal.clone())
        }else {
            None
        }
    }
    
    pub fn as_keyword(&self) -> Option<KeywordType> {
        if let TokenType::Keyword(keyword) = self.token_type {
            Some(keyword.clone())
        }else {
            None
        }
    }

    pub fn is_keyword(&self, comparer: KeywordType) -> bool {
        if let TokenType::Keyword(keyword) = self.token_type {
            keyword == comparer
        }else {
            false
        }
    }
}

impl TreeDump for Token {
    fn print_with_indent(&self, indent: usize, indent_style: &str) {
        print!("{}({}): ", indent_style.repeat(indent), self.start);
        use TokenType::*;
        match &self.token_type {
            Keyword(keyword) => println!("keyword '{}'", keyword),
            Operator(operator) => println!("operator '{}'", operator),
            Identifier(string) => println!("identifier '{}'", string),
            Literal(literal) => {
                use LiteralType::*;
                match literal {
                    _String(string) => println!("literal string '{}'", string),
                    Integer(value) => println!("literal integer '{}'", value),
                    Float(value) => println!("literal float '{}'", value)
                }
            }
        }
    }
}

pub enum LiteralType {
    _String(String),
    Integer(i128),
    Float (f64)
}

impl Clone for LiteralType {
    fn clone(&self) -> LiteralType {
        use LiteralType::*;
        match self {
            _String(text) => {
                _String(text.clone())
            },
            Integer(integer) => {
                Integer(*integer)
            },
            Float(float) => {
                Float(*float)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum OperatorType {
    Add, Subtract, Multiply, Divide, Modulus, Equals
}

impl std::fmt::Display for OperatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use OperatorType::*;
        write!(f, "{}", match self {
            Add => "add",
            Subtract => "sub",
            Multiply => "mult",
            Divide => "div",
            Modulus => "modulus",
            Equals => "equals"
        })
    }
}

pub const OPERATOR_TOKENS: [(&str, OperatorType); 6] = [
    ("==", OperatorType::Equals),
    ("+",  OperatorType::Add),
    ("-",  OperatorType::Subtract),
    ("*",  OperatorType::Multiply),
    ("/",  OperatorType::Divide),
    ("%",  OperatorType::Modulus),
];

#[derive(Copy, Clone, PartialEq)]
pub enum KeywordType {    
    If, While, Loop, As, Run, Assign,
    BlockOpen, BlockClose, BlockSeparator, 
    ArrayOpen, ArrayClose, ArraySeparator
}

impl std::fmt::Display for KeywordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use KeywordType::*;
        write!(f, "{}", match self {
            If => "if",
            While => "while",
            Loop => "loop",
            As => "as",
            Run => "run",
            Assign => "assign",
            BlockOpen => "block open",
            BlockClose => "block close",
            BlockSeparator => "block separator",
            ArrayOpen => "array open",
            ArrayClose => "array close",
            ArraySeparator => "array separator"
        })
    }
}

/// [substr that produces keyword, The keyword enum member, 
/// isAlphabetic(can't be surrounded by other alphabetic things)]
pub const KEYWORD_TOKENS: [(&str, KeywordType, bool); 13] = [
    ("if",      KeywordType::If,                true ),
    ("while",   KeywordType::While,             true ),
    ("loop",    KeywordType::Loop,  	        true ),
    ("as",      KeywordType::As,                true ),
    ("run",     KeywordType::Run,               true ),
    ("=",       KeywordType::Assign,            false),
    ("#(",      KeywordType::BlockOpen,         false),

    (";",       KeywordType::BlockSeparator,    false),
    ("(",       KeywordType::BlockOpen,         false),
    (")",       KeywordType::BlockClose,        false),

    (",",       KeywordType::ArraySeparator,    false),
    ("[",       KeywordType::ArrayOpen,         false),
    ("]",       KeywordType::ArrayClose,        false)
];

/// *IMPORTANT: The needle will change, so buffering the change 
/// with push_state and pop_state around this function is vital*
pub fn try_tokenize_word<'a>(needle: &mut Needle<char>) -> Result<Token, Error> {
    let start = needle.get_index();
    
    while let Some(&c) = needle.peek() {
        if !(c.is_alphabetic() || c == '_') {
            break;
        }else{
            needle.next();
        }
    }

    if start != needle.get_index() {
        return Ok(Token::identifier(
            needle.get_prev_state_index(), 
            needle.get_slice(start, needle.get_index())
            ));
    }
    
    Err(Error::at_needle(needle, 0, "No word found"))
}

/// *IMPORTANT: The needle will change, so buffering the change 
/// with push_state and pop_state around this function is vital 
/// to undo changes if it returned None*
pub fn try_tokenize_string(needle: &mut Needle<char>) -> Result<Token, Error> {
    match needle.read() {
        Some('"') => {},
        _ => {
            return Err(Error::at_needle(needle, 0, "Unexpected start of string, expected '\"'"));
        }
    }

    let mut string = String::new();

    // We turn the needle.read() error into a high priority error
    let mut needle_pos = needle.get_index();
    let mut c = *(needle.read().ok_or(Error::new(needle_pos, 2, "Unexpected end of string"))?);
    while c != '"' {
        if c == '\\' {
            needle_pos = needle.get_index();
            c = *needle.read().ok_or(Error::new(needle_pos, 2, "Unexpected end of string"))?;

            match c {
                '"' => string.push('"'),
                '\\' => string.push('\\'),
                't' => string.push('\t'),
                'n' => string.push('\n'),
                '0' => panic!("TODO: Add hex based character definitions in strings \\0xFA"),
                _ => return Err(Error::new(needle_pos + 1, 1, "Invalid character after '\\'"))
            }
        }else if c == '\n' {
            return Err(Error::at_needle(needle, 2, "Unexpected end of string"));    
        }else{
            string.push(c);
        }

        needle_pos = needle.get_index();
        c = *needle.read().ok_or(Error::new(needle_pos, 2, "Unexpected end of string"))?;
    }

    Ok(Token::literal(
            needle.get_prev_state_index(), 
            LiteralType::_String(string)
        )
    )
}

pub fn try_tokenize_number(needle: &mut Needle<char>) -> Result<Token, Error> {
    let start = needle.get_index();
    let mut value = 0i128;

    while let Some(c) = needle.peek() {
        if let Some(digit) = c.to_digit(10) {
            value = value * 10 + digit as i128;
        }else {
            break;
        }
        
        needle.next();
    }

    // Check for a dot, if there is none, it's an integer
    if !needle.matches_slice(".") {
        // Or well, it didn't move apparently, so it's nothing
        if start == needle.get_index() {
            return Err(Error::at_needle(needle, 0, "Expected a digit or a dot to start of a number"));
        }else {
            return Ok(Token::literal(
                    needle.get_prev_state_index(), 
                    LiteralType::Integer(value)
                ));
        }
    }
    needle.next();

    // Now we know it's a float
    let mut value = value as f64;

    let start = needle.get_index();
    let mut decimal_scalar = 1f64;
    while let Some(c) = needle.peek() {
        if let Some(digit) = c.to_digit(10) {
            decimal_scalar *= 0.1f64;
            value += (digit as f64) * decimal_scalar;
        }else {
            break;
        }
        
        needle.next();
    }

    if start == needle.get_index() {
        return Err(Error::at_needle(needle, 1, 
            "Expected something after '.' to make a float or get a member of a structure"));
    }

    Ok(
        Token::literal(
            needle.get_prev_state_index(), 
            LiteralType::Float(value)
        )
    )
}

fn if_change_err<T>(result: Result<T, Error>, error: &mut Option<Error>) -> Option<T> {
    match result {
        Ok(ok_result) => {
            Some(ok_result)
        },
        Err(result_err) => {
            if let Some(error_) = error {
                if result_err.priority >= error_.priority {
                    *error = Some(result_err);
                }
            }else{
                // It's none, so no priority check required
                *error = Some(result_err);
            }
            None
        }
    }
}

pub fn tokenize(chars: &str) -> (Vec<Token>, Vec<Error>) {
    let mut needle = Needle::from_str(chars, 0usize);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    'outer: loop {
        // Skip the shitespace in the beginning
        loop {
            if let Some(token) = needle.peek() {
                if token.is_whitespace() {
                    needle.next();
                }else{
                    break;
                }
            }else{
                // End of string
                break 'outer;
            }
        }
        
        let mut current_error = None;

        // This one doesn't change the index so we don't have to push/pop
        {
            for op in &OPERATOR_TOKENS {
                if needle.matches_slice(op.0) {
                    tokens.push(Token::operator(needle.get_index(), op.1));
                    needle.skip(op.0.len());
                    continue 'outer;
                }
            }   

            for keyword in &KEYWORD_TOKENS {
                if needle.matches_slice(keyword.0) {
                    if keyword.2 {
                        if needle.match_func_offset(-1, |c| c.is_alphabetic())
                            || needle.match_func_offset(
                                    keyword.0.len() as isize, 
                                    |c| c.is_alphabetic()
                                ) {
                            continue;
                        }
                    }
                    tokens.push(Token::keyword(needle.get_index(), keyword.1));
                    needle.skip(keyword.0.len());
                    continue 'outer;
                }
            }
        }

        needle.push_state();
        if let Some(token) = if_change_err(try_tokenize_string(&mut needle), &mut current_error) {
            tokens.push(token);
            needle.pop_state_no_revert();
            continue;
        }
        needle.pop_state();

        needle.push_state();
        if let Some(token) = if_change_err(try_tokenize_word(&mut needle), &mut current_error) {
            tokens.push(token);
            needle.pop_state_no_revert();
            continue;
        }
        needle.pop_state();

        needle.push_state();
        if let Some(token) = if_change_err(try_tokenize_number(&mut needle), &mut current_error) {
            tokens.push(token);
            needle.pop_state_no_revert();
            continue;
        }
        needle.pop_state();

        if let Some(error) = current_error {
            if error.priority > 0 {
                needle.index = error.loc;
                errors.push(error);
            }else {
                errors.push(
                    Error::at_needle(&needle, 1, "Unexpected token")
                    );
            }
        }else {
            errors.push(
                Error::at_needle(&needle, 1, "Unexpected token")
                );
        }

        needle.next();
    }

    (tokens, errors)
}