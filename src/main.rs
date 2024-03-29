mod tokenizer;
mod lexer;
mod tree_dump;
mod needle;
mod vm;
use needle::{ Needle };
pub use tree_dump::TreeDump;

fn run(code: &String) -> bool {
    let (result, errors, meta) = tokenizer::tokenize(&code[..]);

    if errors.len() > 0 {
        println!("There were errors, woohoo!");
        for error in errors.iter() {
            println!("({}): {}", error.loc, error.msg);
        }
        println!("\n");
    }

    // for token in result.iter() {
    //     token.print();
    // }
    
    let mut scopes = lexer::ScopePool::new();
    let tree = lexer::parse_value(
        &mut Needle::new(result, 0), 
        &meta, 
        scopes.create_scope(), 
        &mut scopes
        );
    
    scopes.print();

    if let Ok(value) = tree {
        value.print();
    }else if let Err(error) = tree {
        println!("Got an error!");
        error.print();
    }

    errors.len() == 0
}

fn main() {
    vm::test_vm();
    return;

    let args: Vec<String> = std::env::args().collect();
    if let Some(data) = args.get(1) {
        match &data[..] {
            "run" => {
                if let Some(data) = args.get(2) {
                    run(data);
                }else {
                    // Open a shell for the to write into
                    println!("\n-- TROLLEDLANG SHELL --\n'quit' or 'exit' to exit the shell\n");
                    loop {
                        match read_line("") {
                            Ok(result) => {
                                if result == "exit" || result == "quit" { break; }
                                print!(">");
                                let result = run(&result);
                                if !result { println!("An error occured :("); }
                            },
                            _ => {
                                println!("Invalid input!");
                            }
                        }
                    }
                }
            },
            "file" => {
                if let Some(path) = args.get(2) {
                    match std::fs::read_to_string(path) {
                        Ok(code) => {
                            let result = run(&code);
                            if !result { println!("An error occured :("); }
                        },
                        Err(err) => {
                            println!("Error opening file! {}", err);
                        }
                    }
                }else{
                    println!("Expected an argument for the path");
                }
            }
            _ => {
                println!("Invalid commandline argument");
            }
        }
    }else {
        println!("Expected a commandline argument");
    }
}

fn read_line<'a>(prompt: &str) -> std::io::Result<String> {
    println!("{}", prompt);
    let buffer = &mut String::new();
    std::io::stdin().read_line(buffer)?; // <- API requires buffer param as of Rust 1.0; returns `Result` of bytes read
    Ok(String::from(buffer.trim_end()))
}