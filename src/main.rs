use pest::Parser;
use rain::interpret::Interpreter;
use rain::parser::{RainParser, Rule};
use rain::typecheck::TypeChecker;
use std::env;
use std::fs;

fn main() {
    // Get argument.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: rain <SOURCE CODE FILE>.");
    }
    let path = &args[1];

    // Read source code.
    let program = fs::read_to_string(path).unwrap();

    // Generate AST.
    let mut pairs = RainParser::parse(Rule::prog, &program).unwrap();
    let expr = RainParser::parse_expression(pairs.next().unwrap().into_inner());

    // Type check.
    let mut type_checker = TypeChecker::new();
    let expr_type = type_checker.type_check(&expr).unwrap();
    println!("Type: {expr_type}");

    // Interpret.
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(&expr).unwrap();
    println!("Value: {result}");
}
