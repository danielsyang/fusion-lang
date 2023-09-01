use std::collections::VecDeque;

use crate::{
    ast::expression::IfExpression,
    lex::{
        lexer::Lexer,
        token::{Token, TokenType},
    },
};

use super::{
    expression::{BooleanLiteral, Identifier, InfixExpression, IntegerLiteral, PrefixExpression},
    statement::{BlockStatement, ExpressionStatement, LetStatement, ReturnStatement},
    tree::{Expression, Statement},
};

#[derive(Clone, Copy, Debug)]
enum Precedence {
    _Int = 0,
    Lowest = 1,
    Equals = 2,
    LessGreater = 3,
    Sum = 4,
    Product = 5,
    Prefix = 6,
    Call = 7,
}

pub struct Parser {
    pub tokens: VecDeque<Token>,
    _errors: Vec<String>,
    current_token: Token,
    pub next_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lex = Lexer::new(input);
        let mut tokens: VecDeque<Token> = VecDeque::new();

        while let Some(token) = lex.next_token() {
            match token.kind {
                TokenType::Whitespace => {}
                _ => tokens.push_back(token),
            }
        }

        let current_token = tokens
            .pop_front()
            .expect("Input did not produce any token.")
            .clone();
        let next_token = tokens.pop_front().expect("Expected at least EOF.").clone();

        Self {
            tokens,
            _errors: vec![],
            current_token,
            next_token,
        }
    }

    pub fn consume_token(&mut self) {
        println!(
            "moved current_token {:?} to {:?}",
            self.current_token.kind, self.next_token.kind
        );
        self.current_token = self.next_token.clone();
        self.next_token = self
            .tokens
            .pop_front()
            .expect("Invalid state, there are no more tokens to consume.");
        println!(
            "moved next_token {:?} to {:?}",
            self.current_token.kind, self.next_token.kind
        );
    }

    pub fn parse_program(&mut self) -> Box<dyn Statement> {
        match self.current_token.kind {
            TokenType::Let => Box::new(self.parse_let_statement()),
            TokenType::Return => Box::new(self.parse_return_statement()),
            _ => Box::new(self.parse_expression_statement()),
        }
    }

    fn expect_next_token(&mut self, kind: TokenType) -> bool {
        if self.next_token.kind == kind {
            self.consume_token();
            return true;
        }
        false
    }

    fn parse_let_statement(&mut self) -> LetStatement {
        let let_token = self.current_token.clone();

        if !self.expect_next_token(TokenType::Identifier) {
            panic!(
                "Expected next token to be TokenType::Identifier, got: {:?}",
                self.next_token.kind
            )
        }

        let identifier = Identifier::new(&self.current_token);

        if !self.expect_next_token(TokenType::Asssign) {
            panic!(
                "Expected next token to be TokenType::Assign, got {:?}",
                self.next_token.kind
            )
        }

        self.consume_token();

        let val = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenType::Semicolon) {
            panic!(
                "Expected next token to be TokenType::Semicolon, got {:?}",
                self.next_token.kind
            )
        }

        LetStatement::new(let_token, identifier, val)
    }

    fn parse_return_statement(&mut self) -> ReturnStatement {
        let return_token = self.current_token.clone();

        self.consume_token();

        let val = self.parse_expression(Precedence::Lowest);

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        ReturnStatement::new(return_token, val)
    }

    fn parse_expression_statement(&mut self) -> ExpressionStatement {
        let curr = self.current_token.clone();
        let exp = self.parse_expression(Precedence::Lowest);
        let stmt = ExpressionStatement::new(curr, exp);

        if self.next_token.kind == TokenType::Semicolon {
            self.consume_token();
        }

        stmt
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let block_token = self.current_token.clone();
        let mut statements: Vec<Box<dyn Statement>> = vec![];

        self.consume_token();

        while self.current_token.kind != TokenType::RightBrace
            && self.current_token.kind != TokenType::Eof
        {
            statements.push(self.parse_program());
            self.consume_token();
        }

        BlockStatement::new(block_token, statements)
    }

    fn parse_prefix_expression(&mut self) -> Box<dyn Expression> {
        let current_prefix_expression = self.current_token.clone();

        self.consume_token();

        let expr = self.parse_expression(Precedence::Prefix);

        let pe = PrefixExpression::new(&current_prefix_expression, expr);

        Box::new(pe)
    }

    fn parse_infix_expression(&mut self, left: Box<dyn Expression>) -> Box<dyn Expression> {
        let curr = self.current_token.clone();

        let precedence = self.current_precedence();
        self.consume_token();

        let right_expression = self.parse_expression(precedence);

        Box::new(InfixExpression::new(&curr, left, right_expression))
    }

    fn parse_grouped_expression(&mut self) -> Box<dyn Expression> {
        self.consume_token();

        let exp = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenType::RightParen) {
            panic!("unexpected next token: TokenType::RightParen")
        }

        exp
    }

    fn parse_if_expression(&mut self) -> Box<dyn Expression> {
        let if_token = self.current_token.clone();

        if !self.expect_next_token(TokenType::LeftParen) {
            panic!(
                "expected token: TokenType::LeftParen, got: {:?}",
                self.next_token.kind
            )
        }

        self.consume_token();

        let condition = self.parse_expression(Precedence::Lowest);

        if !self.expect_next_token(TokenType::RightParen) {
            panic!(
                "expected token: TokenType::RightParen, got: {:?}",
                self.next_token.kind
            )
        }

        if !self.expect_next_token(TokenType::LeftBrace) {
            panic!(
                "expected token: TokenType::LeftBrace, got: {:?}",
                self.next_token.kind
            )
        }

        let consequence = self.parse_block_statement();
        let mut alternative: Option<BlockStatement> = None;

        if self.next_token.kind == TokenType::Else {
            self.consume_token();

            if !self.expect_next_token(TokenType::LeftBrace) {
                panic!(
                    "else: expected token: TokenType::LeftBrace, got {:?}",
                    self.next_token.kind
                )
            }

            alternative = Some(self.parse_block_statement());
        }

        Box::new(IfExpression::new(
            if_token,
            condition,
            consequence,
            alternative,
        ))
    }

    fn parse_expression(&mut self, p: Precedence) -> Box<dyn Expression> {
        let mut left_exp: Box<dyn Expression> = match self.current_token.kind {
            TokenType::Int(v) => Box::new(IntegerLiteral::new(&self.current_token, v)),
            TokenType::Identifier => Box::new(Identifier::new(&self.current_token)),
            TokenType::True => Box::new(BooleanLiteral::new(&self.current_token, true)),
            TokenType::False => Box::new(BooleanLiteral::new(&self.current_token, false)),
            TokenType::BangSign => self.parse_prefix_expression(),
            TokenType::MinusSign => self.parse_prefix_expression(),
            TokenType::LeftParen => self.parse_grouped_expression(),
            TokenType::If => self.parse_if_expression(),
            _ => panic!(
                "parse_expression: not yet implemented, got {:?}",
                self.current_token.kind
            ),
        };

        while (p as u8) < self.next_precedence() && self.next_token.kind != TokenType::Semicolon {
            left_exp = match self.next_token.kind {
                TokenType::PlusSign => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::MinusSign => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::MultiplicationSign => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::SlashSign => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::Eq => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::NotEq => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::LT => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::GT => {
                    self.consume_token();
                    self.parse_infix_expression(left_exp)
                }
                TokenType::LeftParen => {
                    self.consume_token();
                    todo!("parse_call_expression not yet implemented")
                }
                _ => left_exp,
            };
        }

        left_exp
    }

    fn current_precedence(&self) -> Precedence {
        match self.current_token.kind {
            TokenType::Eq => Precedence::Equals,
            TokenType::NotEq => Precedence::Equals,
            TokenType::LT => Precedence::LessGreater,
            TokenType::GT => Precedence::LessGreater,
            TokenType::PlusSign => Precedence::Sum,
            TokenType::MinusSign => Precedence::Sum,
            TokenType::SlashSign => Precedence::Product,
            TokenType::MultiplicationSign => Precedence::Product,
            TokenType::LeftParen => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn next_precedence(&self) -> u8 {
        match self.next_token.kind {
            TokenType::Eq => Precedence::Equals as u8,
            TokenType::NotEq => Precedence::Equals as u8,
            TokenType::LT => Precedence::LessGreater as u8,
            TokenType::GT => Precedence::LessGreater as u8,
            TokenType::PlusSign => Precedence::Sum as u8,
            TokenType::MinusSign => Precedence::Sum as u8,
            TokenType::SlashSign => Precedence::Product as u8,
            TokenType::MultiplicationSign => Precedence::Product as u8,
            TokenType::LeftParen => Precedence::Call as u8,
            _ => Precedence::Lowest as u8,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{
            statement::{ExpressionStatement, LetStatement, ReturnStatement},
            tree::{Node, Statement},
        },
        lex::token::TokenType,
    };

    use super::Parser;

    #[test]
    fn parse_let_statement() {
        let input = "
        let x = 5;
        let y = 100;
        let foobar = y;
        let barfoo = false;
        ";
        let expected = ["let x 5", "let y 100", "let foobar y", "let barfoo false"];

        let mut p = Parser::new(input);

        let mut result: Vec<Box<dyn Statement>> = vec![];
        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            let l = curr.as_any().downcast_ref::<LetStatement>().unwrap();
            assert_eq!(l.string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_return_statement() {
        let input = "
        return 5;
        return 100;
        return foobar;
        ";

        let mut p = Parser::new(input);
        let expected = ["return 5", "return 100", "return foobar"];
        let mut result: Vec<Box<dyn Statement>> = vec![];
        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            let l = curr.as_any().downcast_ref::<ReturnStatement>().unwrap();
            assert_eq!(l.string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_prefix_expression() {
        let input = "
        !5;
        -15;
        !foobar;
        -foobar;
        !true;
        !false;
        ";

        let mut p = Parser::new(input);
        let expression_stmt = [
            "(! 5)",
            "(- 15)",
            "(! foobar)",
            "(- foobar)",
            "(! true)",
            "(! false)",
        ];
        let mut result: Vec<Box<dyn Statement>> = vec![];
        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            let l = curr.as_any().downcast_ref::<ExpressionStatement>().unwrap();
            assert_eq!(
                l.expression.string(),
                expression_stmt.get(i).unwrap().to_string()
            );
        }
    }

    #[test]
    fn parse_infix_expression() {
        let input = "
        5 + 5;
        5 - 5;
        5 * 5;
        5 / 5;
        5 > 5;
        5 < 5;
        5 == 5;
        5 != 5;
        foobar + foobar;
        bar - bar;
        bar * bar;
        true == true;
        false != true;
        5 + 5 * 5;
        -1 + 2;
        a + b * c + d / e - f;
        3 > 5 == false;
        ";

        let mut p = Parser::new(input);
        let expected = [
            "(5 + 5)",
            "(5 - 5)",
            "(5 * 5)",
            "(5 / 5)",
            "(5 > 5)",
            "(5 < 5)",
            "(5 == 5)",
            "(5 != 5)",
            "(foobar + foobar)",
            "(bar - bar)",
            "(bar * bar)",
            "(true == true)",
            "(false != true)",
            "(5 + (5 * 5))",
            "((- 1) + 2)",
            "(((a + (b * c)) + (d / e)) - f)",
            "((3 > 5) == false)",
        ];

        let mut result: Vec<Box<dyn Statement>> = vec![];

        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            assert_eq!(curr.string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_grouped_expression() {
        let input = "
        1 + (2 + 3) + 4;
        (5 + 5) * 2;
        2 / (5 + 5);
        -(5 + 5);
        ";

        let mut p = Parser::new(input);
        let expected = [
            "((1 + (2 + 3)) + 4)",
            "((5 + 5) * 2)",
            "(2 / (5 + 5))",
            "(- (5 + 5))",
        ];

        let mut result: Vec<Box<dyn Statement>> = vec![];

        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            assert_eq!(curr.string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_if_expression() {
        let input = "
        if (x > y) {
            return x;
        }
        ";

        let mut p = Parser::new(input);
        let expected = ["if (x > y) return x"];

        let mut result: Vec<Box<dyn Statement>> = vec![];

        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            assert_eq!(curr.string(), expected.get(i).unwrap().to_string());
        }
    }

    #[test]
    fn parse_if_else_expression() {
        let input = "
        if (x > y) {
            return x;
        } else {
            return y;
        }
        ";

        let mut p = Parser::new(input);
        let expected = ["if (x > y) return x else return y"];

        let mut result: Vec<Box<dyn Statement>> = vec![];

        loop {
            let parsed = p.parse_program();
            result.push(parsed);

            if p.next_token.kind == TokenType::Eof {
                break;
            }
            p.consume_token();
        }

        for (i, curr) in result.iter().enumerate() {
            assert_eq!(curr.string(), expected.get(i).unwrap().to_string());
        }
    }
}
