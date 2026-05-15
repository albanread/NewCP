use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl SourcePosition {
    fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    Identifier,
    Integer,
    Character,
    Real,
    String,
    Symbol,
    Eof,
}

impl TokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::Identifier => "identifier",
            Self::Integer => "integer",
            Self::Character => "character",
            Self::Real => "real",
            Self::String => "string",
            Self::Symbol => "symbol",
            Self::Eof => "eof",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: SourceSpan,
}

impl LexError {
    fn new(message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    pub fn render(&self) -> String {
        format!(
            "{} at {}:{}",
            self.message, self.span.start.line, self.span.start.column
        )
    }
}

pub fn dump_tokens(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(source_text) => match lex_source(&source_text) {
            Ok(tokens) => {
                let rendered = if tokens.is_empty() {
                    "<none>".to_string()
                } else {
                    tokens
                        .iter()
                        .map(render_token)
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                format!(
                    "newcp-lexer token dump\ninput: {}\ntoken-count: {}\n{}",
                    path.display(),
                    tokens.len(),
                    rendered
                )
            }
            Err(error) => format!(
                "newcp-lexer error\ninput: {}\nerror: {}",
                path.display(),
                error.render()
            ),
        },
        Err(error) => format!("newcp-lexer error\ninput: {}\nerror: {}", path.display(), error),
    }
}

pub fn lex_source(source_text: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(source_text).lex_all()
}

struct Lexer<'a> {
    source: &'a str,
    index: usize,
    position: SourcePosition,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            index: 0,
            position: SourcePosition::start(),
        }
    }

    fn lex_all(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            let start = self.position;
            let Some(character) = self.peek_char() else {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    lexeme: "<eof>".to_string(),
                    span: SourceSpan { start, end: start },
                });
                break;
            };

            let token = if is_identifier_start(character) {
                self.lex_identifier_or_keyword()
            } else if character.is_ascii_digit() {
                self.lex_number()?
            } else if character == '"' || character == '\'' {
                self.lex_string()?
            } else {
                self.lex_symbol()?
            };
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        loop {
            while let Some(character) = self.peek_char() {
                if character.is_whitespace() {
                    self.bump_char();
                } else {
                    break;
                }
            }

            if self.peek_char() == Some('(') && self.peek_next_char() == Some('*') {
                self.skip_nested_comment()?;
                continue;
            }

            break;
        }

        Ok(())
    }

    fn skip_nested_comment(&mut self) -> Result<(), LexError> {
        let comment_start = self.position;
        self.bump_char();
        self.bump_char();
        let mut depth = 1usize;

        while let Some(character) = self.peek_char() {
            if character == '(' && self.peek_next_char() == Some('*') {
                self.bump_char();
                self.bump_char();
                depth += 1;
                continue;
            }

            if character == '*' && self.peek_next_char() == Some(')') {
                self.bump_char();
                self.bump_char();
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
                continue;
            }

            self.bump_char();
        }

        Err(LexError::new(
            "unterminated comment",
            SourceSpan {
                start: comment_start,
                end: self.position,
            },
        ))
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        let mut lexeme = String::new();

        while let Some(character) = self.peek_char() {
            if is_identifier_continue(character) {
                lexeme.push(character);
                self.bump_char();
            } else {
                break;
            }
        }

        let kind = if is_keyword(&lexeme) {
            TokenKind::Keyword
        } else {
            TokenKind::Identifier
        };

        Token {
            kind,
            lexeme,
            span: SourceSpan {
                start,
                end: self.position,
            },
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let mut lexeme = String::new();

        while let Some(character) = self.peek_char() {
            if character.is_ascii_digit() {
                lexeme.push(character);
                self.bump_char();
            } else {
                break;
            }
        }

        if self.peek_char() == Some('.') && self.peek_next_char() != Some('.') {
            lexeme.push('.');
            self.bump_char();

            while let Some(character) = self.peek_char() {
                if character.is_ascii_digit() {
                    lexeme.push(character);
                    self.bump_char();
                } else {
                    break;
                }
            }

            if self.peek_char() == Some('E') {
                lexeme.push(self.bump_char().expect("scale marker should exist"));
                if matches!(self.peek_char(), Some('+') | Some('-')) {
                    lexeme.push(self.bump_char().expect("scale sign should exist"));
                }

                let exponent_start = self.position;
                let mut exponent_digits = 0usize;
                while let Some(character) = self.peek_char() {
                    if character.is_ascii_digit() {
                        lexeme.push(character);
                        self.bump_char();
                        exponent_digits += 1;
                    } else {
                        break;
                    }
                }

                if exponent_digits == 0 {
                    return Err(LexError::new(
                        "expected exponent digits after E",
                        SourceSpan {
                            start: exponent_start,
                            end: self.position,
                        },
                    ));
                }
            }

            return Ok(Token {
                kind: TokenKind::Real,
                lexeme,
                span: SourceSpan {
                    start,
                    end: self.position,
                },
            });
        }

        let mut saw_hex_digit_extension = false;
        while let Some(character) = self.peek_char() {
            if is_hex_digit(character) {
                saw_hex_digit_extension = true;
                lexeme.push(character);
                self.bump_char();
            } else {
                break;
            }
        }

        let kind = match self.peek_char() {
            Some('H') => {
                lexeme.push(self.bump_char().expect("suffix should exist"));
                TokenKind::Integer
            }
            Some('L') => {
                lexeme.push(self.bump_char().expect("suffix should exist"));
                TokenKind::Integer
            }
            Some('X') => {
                lexeme.push(self.bump_char().expect("suffix should exist"));
                TokenKind::Character
            }
            Some(character) if is_identifier_start(character) => {
                return Err(LexError::new(
                    format!("invalid numeric literal: {}", lexeme),
                    SourceSpan {
                        start,
                        end: self.position,
                    },
                ));
            }
            _ if saw_hex_digit_extension => {
                return Err(LexError::new(
                    format!("invalid numeric literal: {}", lexeme),
                    SourceSpan {
                        start,
                        end: self.position,
                    },
                ));
            }
            _ => TokenKind::Integer,
        };

        Ok(Token {
            kind,
            lexeme,
            span: SourceSpan {
                start,
                end: self.position,
            },
        })
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let mut lexeme = String::new();
        let quote = self.bump_char().expect("opening quote should exist");
        lexeme.push(quote);

        while let Some(character) = self.peek_char() {
            if character == '\n' || character == '\r' {
                return Err(LexError::new(
                    "unterminated string literal",
                    SourceSpan {
                        start,
                        end: self.position,
                    },
                ));
            }
            lexeme.push(character);
            self.bump_char();

            if character == quote {
                return Ok(Token {
                    kind: TokenKind::String,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.position,
                    },
                });
            }
        }

        Err(LexError::new(
            "unterminated string literal",
            SourceSpan {
                start,
                end: self.position,
            },
        ))
    }

    fn lex_symbol(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let first = self.bump_char().ok_or_else(|| {
            LexError::new(
                "unexpected end of input",
                SourceSpan {
                    start,
                    end: start,
                },
            )
        })?;
        let mut lexeme = String::new();
        lexeme.push(first);

        if let Some(second) = self.peek_char() {
            let is_pair = matches!(
                (first, second),
                (':', '=') | ('<', '=') | ('>', '=') | ('.', '.')
            );
            if is_pair {
                lexeme.push(second);
                self.bump_char();
            }
        }

        if !is_valid_symbol(&lexeme) {
            return Err(LexError::new(
                format!("unexpected character sequence: {}", lexeme),
                SourceSpan {
                    start,
                    end: self.position,
                },
            ));
        }

        Ok(Token {
            kind: TokenKind::Symbol,
            lexeme,
            span: SourceSpan {
                start,
                end: self.position,
            },
        })
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.index..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.source[self.index..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let character = self.peek_char()?;
        self.index += character.len_utf8();
        self.position.offset += character.len_utf8();
        if character == '\n' {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
        Some(character)
    }
}

fn is_identifier_start(character: char) -> bool {
    is_cp_letter(character) || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    is_cp_letter(character) || character.is_ascii_digit() || character == '_'
}

fn is_keyword(upper: &str) -> bool {
    matches!(
        upper,
        "ABSTRACT"
            | "ARRAY"
            | "BEGIN"
            | "BRK"
            | "BY"
            | "CASE"
            | "CLOSE"
            | "CONST"
            | "DIV"
            | "DO"
            | "DEFINITION"
            | "ELSE"
            | "ELSIF"
            | "EMPTY"
            | "END"
            | "EXIT"
            | "EXTENSIBLE"
            | "FOR"
            | "IF"
            | "IMPORT"
            | "IN"
            | "IS"
            | "LIMITED"
            | "LOOP"
            | "MOD"
            | "MODULE"
            | "NIL"
            | "OF"
            | "OR"
            | "OUT"
            | "POINTER"
            | "PROCEDURE"
            | "RECORD"
            | "REPEAT"
            | "RETURN"
            | "THEN"
            | "TO"
            | "TYPE"
            | "UNTIL"
            | "VAR"
            | "WHILE"
            | "WITH"
    )
}

fn is_valid_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "$"
            | "&"
            | "#"
            | "("
            | ")"
            | "*"
            | "+"
            | ","
            | "-"
            | "."
            | ".."
            | "/"
            | ":"
            | ":="
            | ";"
            | "="
            | "<"
            | "<="
            | ">"
            | ">="
            | "^"
            | "["
            | "]"
            | "{"
            | "}"
            | "|"
            | "~"
    )
}

fn is_cp_letter(character: char) -> bool {
    character.is_ascii_alphabetic()
        || matches!(character, '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}' | '\u{00F8}'..='\u{00FF}')
}

fn is_hex_digit(character: char) -> bool {
    matches!(character, 'A'..='F' | '0'..='9')
}

fn render_token(token: &Token) -> String {
    format!(
        "{}:{}@{}:{}-{}:{}",
        token.kind.as_str(),
        token.lexeme,
        token.span.start.line,
        token.span.start.column,
        token.span.end.line,
        token.span.end.column
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_dump_includes_keywords_identifiers_and_spans() {
        let temp = std::env::temp_dir().join("newcp-lexer-test.cp");
        std::fs::write(&temp, "MODULE Demo; IMPORT Kernel; END Demo.")
            .expect("write test module");

        let dump = dump_tokens(&temp);
        let _ = std::fs::remove_file(&temp);

        assert!(dump.contains("keyword:MODULE@1:1-1:7"));
        assert!(dump.contains("identifier:Demo@1:8-1:12"));
        assert!(dump.contains("keyword:IMPORT@1:14-1:20"));
        assert!(dump.contains("eof:<eof>"));
    }

    #[test]
    fn lexer_supports_latin1_identifiers() {
        let tokens = lex_source("VAR Äpfel, façade_1;")
            .expect("lexing should succeed");

        let rendered = tokens
            .iter()
            .map(|token| format!("{}:{}", token.kind.as_str(), token.lexeme))
            .collect::<Vec<_>>();

        assert!(rendered.contains(&"identifier:Äpfel".to_string()));
        assert!(rendered.contains(&"identifier:façade_1".to_string()));
    }

    #[test]
    fn lexer_skips_nested_comments() {
        let tokens = lex_source("MODULE Demo; (* outer (* inner *) outer *) END Demo.")
            .expect("lexing should succeed");

        let lexemes = tokens.iter().map(|token| token.lexeme.as_str()).collect::<Vec<_>>();
        assert_eq!(lexemes, vec!["MODULE", "Demo", ";", "END", "Demo", ".", "<eof>"]);
    }

    #[test]
    fn lexer_handles_numbers_strings_and_multi_char_symbols() {
        let tokens = lex_source("VAR x := 12.5E+1; y := 0FFH; c := 41X; s := 'ok'; a := b..c")
            .expect("lexing should succeed");

        let rendered = tokens
            .iter()
            .map(|token| format!("{}:{}", token.kind.as_str(), token.lexeme))
            .collect::<Vec<_>>();

        assert!(rendered.contains(&"real:12.5E+1".to_string()));
        assert!(rendered.contains(&"integer:0FFH".to_string()));
        assert!(rendered.contains(&"character:41X".to_string()));
        assert!(rendered.contains(&"string:'ok'".to_string()));
        assert!(rendered.contains(&"symbol::=".to_string()));
        assert!(rendered.contains(&"symbol:..".to_string()));
    }

    #[test]
    fn lexer_recognizes_complete_reserved_word_set_samples() {
        let tokens = lex_source("ABSTRACT CLOSE EMPTY EXTENSIBLE LIMITED OUT")
            .expect("lexing should succeed");

        assert!(tokens.iter().all(|token| {
            token.kind == TokenKind::Keyword || token.kind == TokenKind::Eof
        }));
    }

    #[test]
    fn lexer_keeps_export_marks_as_symbols() {
        let tokens = lex_source("VAR value*, readOnly-: INTEGER;")
            .expect("lexing should succeed");

        let rendered = tokens
            .iter()
            .map(|token| format!("{}:{}", token.kind.as_str(), token.lexeme))
            .collect::<Vec<_>>();

        assert!(rendered.contains(&"identifier:value".to_string()));
        assert!(rendered.contains(&"symbol:*".to_string()));
        assert!(rendered.contains(&"identifier:readOnly".to_string()));
        assert!(rendered.contains(&"symbol:-".to_string()));
    }

    #[test]
    fn lexer_reports_invalid_hex_without_suffix() {
        let error = lex_source("VAR x := 0FF;")
            .expect_err("lexing should fail");

        assert!(error.render().contains("invalid numeric literal"));
    }

    #[test]
    fn lexer_reports_unterminated_comment() {
        let error = lex_source("MODULE Demo; (* broken")
            .expect_err("lexing should fail");

        assert!(error.render().contains("unterminated comment"));
    }
}
