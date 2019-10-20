mod string_needle;
use string_needle::{ StringNeedle, TokenError };
use std::vec::Vec;

type Loc = usize;

pub enum Literal {
    _String(String),
    Integer(i128),
    Float (f64)
}

#[derive(Copy, Clone)]
pub enum Operator {
    Add, Subtract, Multiply, Divide, Modulus, Equals
}

pub const OPERATOR_TOKENS: [(&str, Operator); 6] = [
    ("==", Operator::Equals),
    ("+",  Operator::Add),
    ("-",  Operator::Subtract),
    ("*",  Operator::Multiply),
    ("/",  Operator::Divide),
    ("%",  Operator::Modulus),
];

#[derive(Copy, Clone)]
pub enum Keyword {
    If, While, Loop, As, Run, Assign,
    BlockOpen, BlockClose, BlockSeparator, 
    ArrayOpen, ArrayClose, ArraySeparator
}

/// [substr that produces keyword, The keyword enum member, 
/// isAlphabetic(can't be surrounded by other alphabetic things)]
pub const KEYWORD_TOKENS: [(&str, Keyword, bool); 13] = [
    ("if", Keyword::If, true),
    ("while", Keyword::While, true),
    ("loop", Keyword::Loop, true),
    ("as", Keyword::As, true),
    ("run", Keyword::Run, true),
    ("=", Keyword::Assign, false),
    ("#(", Keyword::BlockOpen, false),

    (";", Keyword::BlockSeparator, false),
    ("(", Keyword::BlockOpen, false),
    (")", Keyword::BlockClose, false),

    (",", Keyword::ArraySeparator, false),
    ("[", Keyword::ArrayOpen, false),
    ("]", Keyword::ArrayClose, false)
];

pub enum Token {
    Word(Loc, String),
    _Literal(Loc, Literal),

    Op(Loc, Operator),
    _Keyword(Loc, Keyword)
}

/// *IMPORTANT: The needle will change, so buffering the change 
/// with push_state and pop_state around this function is vital*
pub fn try_tokenize_word<'a>(needle: &mut StringNeedle) -> Result<Token, TokenError> {
    let start = needle.get_index();
    
    while let Some(c) = needle.peek() {
        if !(c.is_alphabetic() || c == '_') {
            break;
        }else{
            needle.next();
        }
    }

    if start != needle.get_index() {
        return Ok(Token::Word(needle.get_prev_state_index(), needle.get_slice(start, needle.get_index())));
    }
    
    Err(TokenError::at_needle(needle, 0, "No word found"))
}

/// *IMPORTANT: The needle will change, so buffering the change 
/// with push_state and pop_state around this function is vital 
/// to undo changes if it returned None*
pub fn try_tokenize_string(needle: &mut StringNeedle) -> Result<Token, TokenError> {
    match needle.read() {
        Ok('"') => {},
        _ => {
            return Err(TokenError::at_needle(needle, 0, "Unexpected start of string, expected '\"'"));
        }
    }

    let mut string = String::new();

    // We turn the needle.read() error into a high priority error
    let mut c = TokenError::if_err_mod(needle.read(), 2, "Unexpected end of string")?;
    while c != '"' {
        if c == '\\' {
            c = TokenError::if_err_mod(needle.read(), 2, "Unexpected end of string")?;

            match c {
                '"' => string.push('"'),
                '\\' => string.push('\\'),
                't' => string.push('\t'),
                'n' => string.push('\n'),
                '0' => panic!("TODO: Add hex based character definitions in strings \\0xFA"),
                _ => return Err(TokenError::at_needle(needle, 1, "Invalid character after '\\'"))
            }
        }else if c == '\n' {
            return Err(TokenError::at_needle(needle, 2, "Unexpected end of string"));    
        }else{
            string.push(c);
        }

        c = TokenError::if_err_mod(needle.read(), 2, "Unexpected end of string")?;
    }

    Ok(Token::_Literal(
            needle.get_prev_state_index(), 
            Literal::_String(string)
            )
        )
}

pub fn try_tokenize_number(needle: &mut StringNeedle) -> Result<Token, TokenError> {
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
            return Err(TokenError::at_needle(needle, 0, "Expected a digit or a dot to start of a number"));
        }else {
            return Ok(
                Token::_Literal(
                    needle.get_prev_state_index(), 
                    Literal::Integer(value)
                    )
                );
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
        return Err(TokenError::at_needle(needle, 1, 
            "Expected something after '.' to make a float or get a member of a structure"));
    }

    Ok(
        Token::_Literal(needle.get_prev_state_index(), 
        Literal::Float(value))
        )
}

fn if_change_err<T>(result: Result<T, TokenError>, error: &mut Option<TokenError>) -> Option<T> {
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

pub fn tokenize(chars: &str) -> (Vec<Token>, Vec<TokenError>) {
    let mut needle = StringNeedle::new(chars, 0usize);
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
                    tokens.push(Token::Op(needle.get_index(), op.1));
                    needle.skip(op.0.len());
                    continue 'outer;
                }
            }   

            for keyword in &KEYWORD_TOKENS {
                if needle.matches_slice(keyword.0) {
                    if keyword.2 {
                        if needle.match_func_offset(-1, char::is_alphabetic, false)
                            || needle.match_func_offset(
                                    keyword.0.len() as isize, 
                                    char::is_alphabetic, false
                                ) {
                            continue;
                        }
                    }
                    tokens.push(Token::_Keyword(needle.get_index(), keyword.1));
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
                    TokenError::at_needle(&needle, 1, "Unexpected token")
                    );
            }
        }else {
            errors.push(
                TokenError::at_needle(&needle, 1, "Unexpected token")
                );
        }

        needle.next();
    }

    (tokens, errors)
}

pub fn dump_token(token: &Token) {
    use Token::*;
    match token {
        Word(loc, word) => {
            println!("({}): Word '{}'", loc, word);
        },
        _Literal(loc, data) => {
            use Literal::*;
            println!("({}): Literal {}", loc, match data {
                _String(string) => format!("string \"{}\"", string),
                Integer(integer) => format!("integer '{}'", integer),
                Float(float) => format!("float '{}'", float)
            });
        },
        Op(loc, op) => {
            use Operator::*;
            println!("({}): Operator '{}'", loc, match op {
                Add => "add",
                Subtract => "subtract",
                Multiply => "multiply",
                Divide => "divide",
                Modulus => "modulus",
                Equals => "equals"
            });
        },
        _Keyword(loc, keyword) => {
            use Keyword::*;
            println!("({}): Keyword '{}'", loc, match keyword {
                If => "if",
                As => "as",
                Assign => "assign",

                While => "while",
                Loop => "loop",
                Run => "run",
                
                BlockOpen => "block open",
                BlockClose => "block close",
                BlockSeparator => "block separator",

                ArrayOpen => "array open",
                ArrayClose => "array close",
                ArraySeparator => "array separator"
            });
        }
    }
}