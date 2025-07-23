use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Fn,
    Type,
    Class,
    Import,
    Let,
    Return,
    Prompt,
    Meaning,

    // Literals
    StringLit,
    IntLit,
    FloatLit,
    BoolLit,
    Identifier,

    // Symbols
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftAngle,
    RightAngle,
    Semicolon,
    Colon,
    Equals,
    Comma,
    Arrow,

    // Special
    Eof,
    Error,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        let tokens = Self::tokenize(input)?;
        Ok(Self { tokens, current: 0 })
    }

    pub fn parse(&mut self) -> Result<AstNode> {
        self.parse_program()
    }

    fn parse_program(&mut self) -> Result<AstNode> {
        let mut program = AstNode::new(AstNodeType::Program);

        while !self.is_at_end() {
            match self.parse_declaration() {
                Ok(decl) => program.add_child(decl),
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    self.synchronize();
                }
            }
        }

        Ok(program)
    }

    fn parse_declaration(&mut self) -> Result<AstNode> {
        match self.peek().token_type {
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::Type => self.parse_type_declaration(),
            TokenType::Class => self.parse_class_declaration(),
            TokenType::Import => self.parse_import_declaration(),
            _ => Err(anyhow!("Expected declaration at line {}", self.peek().line)),
        }
    }

    fn parse_function_declaration(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Fn)?;

        let name = self.consume_identifier()?;
        let mut func = AstNode::new(AstNodeType::FunctionDecl);
        func.set_string("name", &name);

        self.consume(&TokenType::LeftParen)?;

        if !self.check(&TokenType::RightParen) {
            let params = self.parse_parameter_list()?;
            func.add_child(params);
        }

        self.consume(&TokenType::RightParen)?;

        // Optional return type
        if self.match_token(&TokenType::Arrow) {
            let return_type = self.parse_type()?;
            func.add_child(return_type);
        }

        let body = self.parse_block()?;
        let mut func_body = AstNode::new(AstNodeType::FunctionBody);
        for child in body.children {
            func_body.children.push(child);
        }
        func.add_child(func_body);

        Ok(func)
    }

    fn parse_type_declaration(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Type)?;
        let name = self.consume_identifier()?;
        self.consume(&TokenType::Equals)?;
        let type_def = self.parse_type()?;
        self.consume(&TokenType::Semicolon)?;

        let mut type_decl = AstNode::new(AstNodeType::TypeDecl);
        type_decl.set_string("name", &name);
        type_decl.add_child(type_def);

        Ok(type_decl)
    }

    fn parse_type(&mut self) -> Result<AstNode> {
        if self.match_token(&TokenType::Meaning) {
            self.parse_meaning_type()
        } else {
            self.parse_basic_type()
        }
    }

    fn parse_meaning_type(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::LeftAngle)?;
        let base_type = self.parse_type()?;
        self.consume(&TokenType::RightAngle)?;
        self.consume(&TokenType::LeftParen)?;
        let meaning = self.consume_string_literal()?;
        self.consume(&TokenType::RightParen)?;

        let mut meaning_type = AstNode::new(AstNodeType::MeaningType);
        meaning_type.set_string("meaning", &meaning);
        meaning_type.add_child(base_type);

        Ok(meaning_type)
    }

    fn parse_basic_type(&mut self) -> Result<AstNode> {
        let name = self.consume_identifier()?;
        let mut basic_type = AstNode::new(AstNodeType::BasicType);
        basic_type.set_string("type", &name);
        Ok(basic_type)
    }

    fn parse_prompt_statement(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Prompt)?;
        let template = self.consume_string_literal()?;
        self.consume(&TokenType::Semicolon)?;

        let mut prompt = AstNode::new(AstNodeType::PromptBlock);
        prompt.set_string("template", &template);

        Ok(prompt)
    }

    // Helper methods
    fn tokenize(input: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();
        let mut line = 1;
        let mut column = 1;

        while let Some((pos, ch)) = chars.next() {
            match ch {
                ' ' | '\t' | '\r' => column += 1,
                '\n' => {
                    line += 1;
                    column = 1;
                }
                '(' => tokens.push(Token {
                    token_type: TokenType::LeftParen,
                    value: "(".to_string(),
                    line,
                    column,
                }),
                ')' => tokens.push(Token {
                    token_type: TokenType::RightParen,
                    value: ")".to_string(),
                    line,
                    column,
                }),
                '{' => tokens.push(Token {
                    token_type: TokenType::LeftBrace,
                    value: "{".to_string(),
                    line,
                    column,
                }),
                '}' => tokens.push(Token {
                    token_type: TokenType::RightBrace,
                    value: "}".to_string(),
                    line,
                    column,
                }),
                '<' => tokens.push(Token {
                    token_type: TokenType::LeftAngle,
                    value: "<".to_string(),
                    line,
                    column,
                }),
                '>' => tokens.push(Token {
                    token_type: TokenType::RightAngle,
                    value: ">".to_string(),
                    line,
                    column,
                }),
                ';' => tokens.push(Token {
                    token_type: TokenType::Semicolon,
                    value: ";".to_string(),
                    line,
                    column,
                }),
                ':' => tokens.push(Token {
                    token_type: TokenType::Colon,
                    value: ":".to_string(),
                    line,
                    column,
                }),
                '=' => tokens.push(Token {
                    token_type: TokenType::Equals,
                    value: "=".to_string(),
                    line,
                    column,
                }),
                ',' => tokens.push(Token {
                    token_type: TokenType::Comma,
                    value: ",".to_string(),
                    line,
                    column,
                }),
                '"' => {
                    let mut string_val = String::new();
                    while let Some((_, ch)) = chars.next() {
                        if ch == '"' {
                            break;
                        }
                        string_val.push(ch);
                    }
                    tokens.push(Token {
                        token_type: TokenType::StringLit,
                        value: string_val,
                        line,
                        column,
                    });
                }
                '-' if chars.peek() == Some(&(pos + 1, '>')) => {
                    chars.next(); // consume '>'
                    tokens.push(Token {
                        token_type: TokenType::Arrow,
                        value: "->".to_string(),
                        line,
                        column,
                    });
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut identifier = String::new();
                    identifier.push(c);

                    while let Some(&(_, ch)) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            identifier.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    let token_type = match identifier.as_str() {
                        "fn" => TokenType::Fn,
                        "type" => TokenType::Type,
                        "class" => TokenType::Class,
                        "import" => TokenType::Import,
                        "let" => TokenType::Let,
                        "return" => TokenType::Return,
                        "prompt" => TokenType::Prompt,
                        "Meaning" => TokenType::Meaning,
                        "true" | "false" => TokenType::BoolLit,
                        _ => TokenType::Identifier,
                    };

                    tokens.push(Token {
                        token_type,
                        value: identifier,
                        line,
                        column,
                    });
                }
                c if c.is_ascii_digit() => {
                    let mut number = String::new();
                    number.push(c);

                    let mut is_float = false;
                    while let Some(&(_, ch)) = chars.peek() {
                        if ch.is_ascii_digit() {
                            number.push(ch);
                            chars.next();
                        } else if ch == '.' && !is_float {
                            number.push(ch);
                            chars.next();
                            is_float = true;
                        } else {
                            break;
                        }
                    }

                    let token_type = if is_float {
                        TokenType::FloatLit
                    } else {
                        TokenType::IntLit
                    };

                    tokens.push(Token {
                        token_type,
                        value: number,
                        line,
                        column,
                    });
                }
                _ => {
                    // Skip unknown characters for now
                }
            }
            column += 1;
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            value: String::new(),
            line,
            column,
        });

        Ok(tokens)
    }

    fn consume(&mut self, expected: &TokenType) -> Result<()> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!(
                "Expected {:?}, found {:?}",
                expected,
                self.peek().token_type
            ))
        }
    }

    fn consume_identifier(&mut self) -> Result<String> {
        if self.check(&TokenType::Identifier) {
            let token = self.advance();
            Ok(token.value.clone())
        } else {
            Err(anyhow!("Expected identifier"))
        }
    }

    fn consume_string_literal(&mut self) -> Result<String> {
        if self.check(&TokenType::StringLit) {
            let token = self.advance();
            Ok(token.value.clone())
        } else {
            Err(anyhow!("Expected string literal"))
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class | TokenType::Fn | TokenType::Let | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    // Additional parsing methods would be implemented here following the same pattern
    fn parse_parameter_list(&mut self) -> Result<AstNode> {
        let mut params = AstNode::new(AstNodeType::ParamList);

        loop {
            let param = self.parse_parameter()?;
            params.add_child(param);

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_parameter(&mut self) -> Result<AstNode> {
        let name = self.consume_identifier()?;
        self.consume(&TokenType::Colon)?;
        let param_type = self.parse_type()?;

        let mut param = AstNode::new(AstNodeType::Parameter);
        param.set_string("name", &name);
        param.add_child(param_type);

        Ok(param)
    }

    fn parse_block(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::LeftBrace)?;
        let mut block = AstNode::new(AstNodeType::Block);

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => block.add_child(stmt),
                Err(e) => {
                    eprintln!("Statement parse error: {}", e);
                    self.synchronize();
                }
            }
        }

        self.consume(&TokenType::RightBrace)?;
        Ok(block)
    }

    fn parse_statement(&mut self) -> Result<AstNode> {
        match self.peek().token_type {
            TokenType::Let => self.parse_variable_declaration(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Prompt => self.parse_prompt_statement(),
            TokenType::LeftBrace => self.parse_block(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Let)?;
        let name = self.consume_identifier()?;

        let mut var_decl = AstNode::new(AstNodeType::VarDecl);
        var_decl.set_string("name", &name);

        // Optional type annotation
        if self.match_token(&TokenType::Colon) {
            let var_type = self.parse_type()?;
            var_decl.add_child(var_type);
        }

        self.consume(&TokenType::Equals)?;
        let init_expr = self.parse_expression()?;
        var_decl.add_child(init_expr);

        self.consume(&TokenType::Semicolon)?;
        Ok(var_decl)
    }

    fn parse_return_statement(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Return)?;
        let mut ret_stmt = AstNode::new(AstNodeType::ReturnStmt);

        if !self.check(&TokenType::Semicolon) {
            let expr = self.parse_expression()?;
            ret_stmt.add_child(expr);
        }

        self.consume(&TokenType::Semicolon)?;
        Ok(ret_stmt)
    }

    fn parse_expression_statement(&mut self) -> Result<AstNode> {
        let expr = self.parse_expression()?;
        self.consume(&TokenType::Semicolon)?;

        let mut expr_stmt = AstNode::new(AstNodeType::ExprStmt);
        expr_stmt.add_child(expr);

        Ok(expr_stmt)
    }

    fn parse_expression(&mut self) -> Result<AstNode> {
        self.parse_call_expression()
    }

    fn parse_call_expression(&mut self) -> Result<AstNode> {
        let mut expr = self.parse_primary()?;

        while self.match_token(&TokenType::LeftParen) {
            let mut call = AstNode::new(AstNodeType::CallExpr);
            if let Some(name) = expr.get_string("name") {
                call.set_string("function", name);
            }

            if !self.check(&TokenType::RightParen) {
                let args = self.parse_argument_list()?;
                for child in args.children {
                    call.children.push(child);
                }
            }

            self.consume(&TokenType::RightParen)?;
            expr = call;
        }

        Ok(expr)
    }

    fn parse_argument_list(&mut self) -> Result<AstNode> {
        let mut args = AstNode::new(AstNodeType::ParamList);

        loop {
            let arg = self.parse_expression()?;
            args.add_child(arg);

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<AstNode> {
        let token = self.advance().clone();

        match token.token_type {
            TokenType::StringLit => {
                let mut node = AstNode::new(AstNodeType::StringLiteral);
                node.set_string("value", &token.value);
                Ok(node)
            }
            TokenType::IntLit => {
                let mut node = AstNode::new(AstNodeType::IntLiteral);
                node.set_int("value", token.value.parse().unwrap_or(0));
                Ok(node)
            }
            TokenType::FloatLit => {
                let mut node = AstNode::new(AstNodeType::FloatLiteral);
                node.set_float("value", token.value.parse().unwrap_or(0.0));
                Ok(node)
            }
            TokenType::BoolLit => {
                let mut node = AstNode::new(AstNodeType::BoolLiteral);
                node.set_bool("value", &token.value == "true");
                Ok(node)
            }
            TokenType::Identifier => {
                let mut node = AstNode::new(AstNodeType::Identifier);
                node.set_string("name", &token.value);
                Ok(node)
            }
            _ => Err(anyhow!(
                "Unexpected token in expression: {:?}",
                token.token_type
            )),
        }
    }

    fn parse_class_declaration(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Class)?;
        let name = self.consume_identifier()?;
        self.consume(&TokenType::LeftBrace)?;

        let mut class = AstNode::new(AstNodeType::ClassDecl);
        class.set_string("name", &name);

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.parse_class_member() {
                Ok(member) => class.add_child(member),
                Err(e) => {
                    eprintln!("Class member parse error: {}", e);
                    self.synchronize();
                }
            }
        }

        self.consume(&TokenType::RightBrace)?;
        Ok(class)
    }

    fn parse_class_member(&mut self) -> Result<AstNode> {
        match self.peek().token_type {
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::Identifier => self.parse_member_variable(),
            _ => Err(anyhow!("Expected class member")),
        }
    }

    fn parse_member_variable(&mut self) -> Result<AstNode> {
        let name = self.consume_identifier()?;
        self.consume(&TokenType::Colon)?;
        let var_type = self.parse_type()?;
        self.consume(&TokenType::Semicolon)?;

        let mut var = AstNode::new(AstNodeType::MemberVar);
        var.set_string("name", &name);
        var.add_child(var_type);

        Ok(var)
    }

    fn parse_import_declaration(&mut self) -> Result<AstNode> {
        self.consume(&TokenType::Import)?;
        let path = self.consume_string_literal()?;
        self.consume(&TokenType::Semicolon)?;

        let mut import = AstNode::new(AstNodeType::Import);
        import.set_string("path", &path);

        Ok(import)
    }
}

pub fn parse_string(source: &str) -> Result<AstNode> {
    let mut parser = Parser::new(source)?;
    parser.parse()
}
