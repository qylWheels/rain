use pest::Parser;
use rain::interpret::Interpreter;
use rain::parser::{RainParser, Rule};
use rain::typecheck::TypeChecker;
use std::fs;

fn main() {
    // Read source code.
    let program = fs::read_to_string("src/test.rain").unwrap();

    // Generate AST.
    let mut pairs = RainParser::parse(Rule::prog, &program).unwrap();
    let expr = RainParser::parse_expression(pairs.next().unwrap().into_inner());
    println!("ast = {expr:#?}");

    // Type check.
    let mut type_checker = TypeChecker::new();
    let expr_type = type_checker.type_check(&expr).unwrap();
    println!("type = {expr_type}");

    // Interpret.
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(&expr).unwrap();
    println!("result = {result:#?}");
}
