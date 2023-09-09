use crate::ast::statement::Block;

use self::{environment::Environment, object::Object};

pub mod environment;
pub mod object;
pub mod program;

pub fn eval_block(block: &Block, env: &mut Environment) -> Object {
    let mut result = Object::None;

    for sttm in block {
        match sttm.eval(env) {
            Object::Return(r) => return Object::Return(r),
            _ => result = sttm.eval(env),
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::ast::parser::Parser;

    use super::environment::Environment;

    #[test]
    fn eval_integer_expression() {
        let mut env = Environment::new();
        let inputs = [
            "5;",
            "10;",
            "-10;",
            "-5;",
            "5 + 5 + 5 + 5 - 10;",
            "2 * 2 * 2 * 2 * 2;",
            "50 / 2 * 2 + 10;",
            "3 * (3 * 3) + 10;",
            "(5 + 10 * 2 + 15 / 3) * 2 + -10;",
        ];
        let expected = ["5", "10", "-10", "-5", "10", "32", "60", "37", "50"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_boolean_expression() {
        let mut env = Environment::new();
        let inputs = [
            "true;",
            "false;",
            "1 < 2;",
            "1 > 2;",
            "1 == 2;",
            "1 != 2;",
            "true == true;",
            "true != true;",
            "1 + 2 == 3;",
            "1 + 2 == 2 + 1;",
        ];
        let expected = [
            "true", "false", "true", "false", "false", "true", "true", "false", "true", "true",
        ];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_bang_expression() {
        let mut env = Environment::new();
        let inputs = ["!true;", "!false;", "!!true;", "!!false;"];
        let expected = ["false", "true", "true", "false"];
        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_if_else_expression() {
        let mut env = Environment::new();
        let inputs = [
            "if (true) { 10 };",
            "if (true) { 10 } else { 20 };",
            "if (false) { 10 } else { 20 }",
            "if (false) { 10 };",
        ];
        let expected = ["10", "10", "20", "None"];
        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_return_statement() {
        let mut env = Environment::new();
        let inputs = [
            "return 10;",
            "return 2 * 5;",
            "return 2 * 5; 9;",
            "
            if (10 > 1) {
                if (10 > 1) {
                    return 10;
                }
                return 1;
            }
            ",
        ];
        let expected = ["10", "10", "10", "10"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_let_statements() {
        let mut env = Environment::new();
        let inputs = [
            "let a = 5; a;",
            "let a = 5 * 5; a;",
            "let a = 5; let b = a; b;",
            "let a = 5; let b = a; let c = a + b + 5; c;",
            "foobar;",
        ];
        let expected = ["5", "25", "5", "15", "error: identifier not found: foobar"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_block() {
        let mut env = Environment::new();
        let inputs = ["fn abc(x) { x + 2; };"];
        let expected = ["Fn abc ( x ) { + Left Ident (x) , Right Number (2) }"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_application() {
        let mut env = Environment::new();
        let inputs = [
            "fn abc(x) { return x * x; }; abc(5);",
            "fn add(x, y) { return x + y; }; add(5 + 5, add(5, 5));",
        ];
        let expected = ["25", "20"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn eval_function_closures() {
        let mut env = Environment::new();
        let inputs = ["fn abc(x) { fn inner(y) { x + y; }; }; let first = abc(2); first(2)"];
        let expected = ["4"];

        for (i, input) in inputs.iter().enumerate() {
            let mut p = Parser::new(input);
            let program = p.build_ast();
            let result = program.eval_statements(&mut env);
            assert_eq!(result.to_string(), expected.get(i).unwrap().to_string());
        }
    }
}