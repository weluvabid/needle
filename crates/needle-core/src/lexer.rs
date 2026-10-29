use crate::{Diagnostic, DiagnosticCode, SourceSpan};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    Word(String),
    Number(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

pub(crate) fn tokenize(source: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer {
        source,
        byte: 0,
        line: 1,
        column: 1,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
    };
    lexer.run();
    (lexer.tokens, lexer.diagnostics)
}

struct Lexer<'a> {
    source: &'a str,
    byte: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer<'_> {
    fn run(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
                continue;
            }

            if ch == '#' {
                while self.peek().is_some_and(|value| value != '\n') {
                    self.bump();
                }
                continue;
            }

            if ch.is_ascii_alphabetic() || ch == '_' {
                self.word();
                continue;
            }

            if ch.is_ascii_digit() {
                self.number();
                continue;
            }

            let (start, line, column) = (self.byte, self.line, self.column);
            self.bump();
            self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnknownCharacter,
                format!("Loose thread: unexpected character {ch:?}."),
                self.span(start, line, column),
            ));
        }
    }

    fn word(&mut self) {
        let (start, line, column) = (self.byte, self.line, self.column);
        while self
            .peek()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            self.bump();
        }
        self.tokens.push(Token {
            kind: TokenKind::Word(self.source[start..self.byte].to_owned()),
            span: self.span(start, line, column),
        });
    }

    fn number(&mut self) {
        let (start, line, column) = (self.byte, self.line, self.column);
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump();
        }

        let span = self.span(start, line, column);
        match self.source[start..self.byte].parse::<u32>() {
            Ok(value) => self.tokens.push(Token {
                kind: TokenKind::Number(value),
                span,
            }),
            Err(_) => self.diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidCount,
                "That count is too large to count safely.",
                span,
            )),
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.byte..)?.chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.byte += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn span(&self, start: usize, line: usize, column: usize) -> SourceSpan {
        SourceSpan::new(start, self.byte, line, column, self.line, self.column)
    }
}
