use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

/// 位置情報。.0から.1までの区間を表す
/// たとえばLoc(4, 6)なら入力文字の5文字目から7文字目までの区間を表す(0始まり)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Loc(usize, usize);

// loc に便利メソッドを実装しておく
impl Loc {
    fn merge(&self, other: &Loc) -> Loc {
        use std::cmp::{max, min};
        Loc(min(self.0, other.0), max(self.1, other.1))
    }
}

/// アノテーション。値にさまざまなデータを持たせたもの。ここではLocを持たせている
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Annot<T> {
    value: T,
    loc: Loc,
}

impl<T> Annot<T> {
    fn new(value: T, loc: Loc) -> Self {
        Self { value, loc }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TokenKind {
    /// [0-9][0-9]*
    Number(u64),
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Asterisk,
    /// /
    Slash,
    /// (
    LParen,
    /// )
    RParen,
}

// TokenKindにアノテーションを付けたものをTokenとして定義しておく
type Token = Annot<TokenKind>;

// ヘルパーメソッドを定義しておく
impl Token {
    fn number(n: u64, loc: Loc) -> Self {
        Self::new(TokenKind::Number(n), loc)
    }
    fn plus(loc: Loc) -> Self {
        Self::new(TokenKind::Plus, loc)
    }
    fn minus(loc: Loc) -> Self {
        Self::new(TokenKind::Minus, loc)
    }
    fn asterisk(loc: Loc) -> Self {
        Self::new(TokenKind::Asterisk, loc)
    }
    fn slash(loc: Loc) -> Self {
        Self::new(TokenKind::Slash, loc)
    }
    fn lparen(loc: Loc) -> Self {
        Self::new(TokenKind::LParen, loc)
    }
    fn rparen(loc: Loc) -> Self {
        Self::new(TokenKind::RParen, loc)
    }
}

// TokenKindと同様の実装をする
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LexErrorKind {
    InvalidChar(char),
    Eof,
}

type LexError = Annot<LexErrorKind>;

impl LexError {
    fn invalid_char(c: char, loc: Loc) -> Self {
        Self::new(LexErrorKind::InvalidChar(c), loc)
    }
    fn eof(loc: Loc) -> Self {
        Self::new(LexErrorKind::Eof, loc)
    }
}

/// 字句解析器
fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    // 解析結果を保存するベクタ
    let mut tokens = Vec::new();
    // 入力
    let input = input.as_bytes();
    // 位置を管理する値
    let mut pos = 0;
    // サブレキサを呼んだ後posを更新するマクロ
    macro_rules! lex_a_token {
        ($lexer:expr) => {{
            let (tok, p) = $lexer?;
            tokens.push(tok);
            pos = p;
        }};
    }
    while pos < input.len() {
        // ここでそれぞれの関数にinputとposを渡す
        match input[pos] {
            // 遷移図通りの実装
            b'0'...b'9' => lex_a_token!(lex_number(input, pos)),
            b'+' => lex_a_token!(lex_plus(input, pos)),
            b'-' => lex_a_token!(lex_minus(input, pos)),
            b'*' => lex_a_token!(lex_asterisk(input, pos)),
            b'/' => lex_a_token!(lex_slash(input, pos)),
            b'(' => lex_a_token!(lex_lparen(input, pos)),
            b')' => lex_a_token!(lex_rparen(input, pos)),
            // 空白を扱う
            b' ' | b'\n' | b'\t' => {
                let ((), p) = skip_spaces(input, pos)?;
                pos = p;
            }
            // それ以外が来たらエラー
            b => return Err(LexError::invalid_char(b as char, Loc(pos, pos + 1))),
        }
    }
    Ok(tokens)
}

/// posのバイトが期待するものであれば1バイト消費してposを1進める
fn consume_byte(input: &[u8], pos: usize, b: u8) -> Result<(u8, usize), LexError> {
    // posが入力サイズ以上なら入力が終わっている
    // 1バイト期待しているのに終わっているのでエラー
    if input.len() <= pos {
        return Err(LexError::eof(Loc(pos, pos)));
    }
    // 入力が期待するものでなければエラー
    if input[pos] != b {
        return Err(LexError::invalid_char(
            input[pos] as char,
            Loc(pos, pos + 1),
        ));
    }
    Ok((b, pos + 1))
}

fn lex_plus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
// Result::mapを使うことで結果が正常だった場合の処理を簡潔に書ける
// これはこのコードと等価
// ```
// match consume_byte(input, start, b'+') {
//     Ok((_, end)) => (Token::plus(Loc(start, end)), end),
    //     Err(err) => Err(err),
// }
    consume_byte(input, start, b'+').map(|(_, end)| (Token::plus(Loc(start, end)), end))
}
fn lex_minus(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'-').map(|(_, end)| (Token::minus(Loc(start, end)), end))
}
fn lex_asterisk(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'*').map(|(_, end)| (Token::asterisk(Loc(start, end)), end))
}
fn lex_slash(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'/').map(|(_, end)| (Token::slash(Loc(start, end)), end))
}
fn lex_lparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b'(').map(|(_, end)| (Token::lparen(Loc(start, end)), end))
}
fn lex_rparen(input: &[u8], start: usize) -> Result<(Token, usize), LexError> {
    consume_byte(input, start, b')').map(|(_, end)| (Token::rparen(Loc(start, end)), end))
}

fn lex_number(input: &[u8], mut pos: usize) -> Result<(Token, usize), LexError> {
    use std::str::from_utf8;

    let start = pos;
    // recognize_manyを使って数値を読み込む
    let end = recognize_many(input, start, |b| b"1234567890".contains(&b));
    let n = from_utf8(&input[start..end])
        // start..endの構成からfrom_utf8は常に成功する
        .unwrap()
        .parse()
        // 同じく構成からfrom_utf8は常に成功するためunwrapしても安全
        .unwrap();
    Ok((Token::number(n, Loc(start, end)), end))
}

fn skip_spaces(input: &[u8], mut pos: usize) -> Result<((), usize), LexError> {
    // 入力に空白文字が続く限り位置を進める
    let pos = recognize_many(input, pos, |b|  b" \n\t".contains(&b));
    Ok(((), pos))
}

fn recognize_many(input: &[u8], mut pos: usize, mut f: impl FnMut(u8) -> bool) -> usize {
    while pos < input.len() && f(input[pos]) {
        pos += 1;
    }
    pos
}

#[test]
fn test_lexer() {
    assert_eq!(
        lex("1 + 2 * 3 - -10"),
        Ok(vec![
            Token::number(1, Loc(0, 1)),
            Token::plus(Loc(2, 3)),
            Token::number(2, Loc(4, 5)),
            Token::asterisk(Loc(6, 7)),
            Token::number(3, Loc(8, 9)),
            Token::minus(Loc(10, 11)),
            Token::minus(Loc(12, 13)),
            Token::number(10, Loc(13, 15)),
        ])
    )
}

/// ASTを表すデータ型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AstKind {
    /// 数値
    Num(u64),
    /// 単項演算
    UniOp { op: UniOp, e: Box<Ast> },
    /// 二項演算
    BinOp { op: BinOp, l: Box<Ast>, r: Box<Ast> },
}

type Ast = Annot<AstKind>;
// ヘルパメソッドを定義しておく
impl Ast {
    fn num(n: u64, loc: Loc) -> Self {
// impl<T> Annot<T>で実装したnewを呼ぶ
        Self::new(AstKind::Num(n), loc)
    }
    fn uniop(op: UniOp, e: Ast, loc: Loc) -> Self {
        Self::new(AstKind::UniOp { op, e: Box::new(e) }, loc)
    }
    fn binop(op: BinOp, l: Ast, r: Ast, loc: Loc) -> Self {
        Self::new(
            AstKind::BinOp {
                op,
                l: Box::new(l),
                r: Box::new(r),
            },
            loc,
        )
    }
}

/// 単項演算子を表すデータ型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum UniOpKind {
    /// 正号
    Plus,
    /// 負号
    Minus,
}
type UniOp = Annot<UniOpKind>;
impl UniOp {
    fn plus(loc: Loc) -> Self {
        Self::new(UniOpKind::Plus, loc)
    }
    fn minus(loc: Loc) -> Self {
        Self::new(UniOpKind::Minus, loc)
    }
}
/// 二項演算子を表すデータ型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BinOpKind {
    /// 加算
    Add,
    /// 減算
    Sub,
    /// 乗算
    Mult,
    /// 除算
    Div,
}
type BinOp = Annot<BinOpKind>;
impl BinOp {
    fn add(loc: Loc) -> Self {
        Self::new(BinOpKind::Add, loc)
    }
    fn sub(loc: Loc) -> Self {
        Self::new(BinOpKind::Sub, loc)
    }
    fn mult(loc: Loc) -> Self {
        Self::new(BinOpKind::Mult, loc)
    }
    fn div(loc: Loc) -> Self {
        Self::new(BinOpKind::Div, loc)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ParseError {
    /// 予期しないトークンがきた
    UnexpectedToken(Token),
    /// 式を期待していたのに式でないものがきた
    NotExpression(Token),
    /// 演算子を期待していたのに演算子でないものがきた
    NotOperator(Token),
    /// 括弧が閉じられていない
    UnclosedOpenParen(Token),
    /// 式の解析が終わったのにまだトークンが残っている
    RedundantExpression(Token),
    /// パース途中で入力が終わった
    Eof,
}

use std::iter::Peekable;

fn parse(tokens: Vec<Token>) -> Result<Ast, ParseError> {
// 入力をイテレータにし、Peekableにする
    let mut tokens = tokens.into_iter().peekable();
// その後parse_exprを呼んでエラー処理をする
    let ret = parse_expr(&mut tokens)?;
    match tokens.next() {
        Some(tok) => Err(ParseError::RedundantExpression(tok)),
        None => Ok(ret),
    }
}
fn parse_expr<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
//parse_exprは parse_expr3を呼ぶだけ
    parse_expr3(tokens)
}
fn parse_expr3<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
    // parse_left_binopに渡す関数を定義する
    fn parse_expr3_op<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<BinOp, ParseError>
        where
            Tokens: Iterator<Item = Token>,
    {
        let op = tokens
            .peek()
// イテレータの終わりは入力の終端なのでエラーを出す
            .ok_or(ParseError::Eof)
// エラーを返すかもしれない値をつなげる
            .and_then(|tok| match tok.value {
                TokenKind::Plus => Ok(BinOp::add(tok.loc.clone())),
                TokenKind::Minus => Ok(BinOp::sub(tok.loc.clone())),
                _ => Err(ParseError::NotOperator(tok.clone())),
            })?;
        tokens.next();
        Ok(op)
    }
    parse_left_binop(tokens, parse_expr2, parse_expr3_op)
}

fn parse_expr2<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
    let mut e = parse_expr1(tokens)?;
    loop {
        match tokens.peek().map(|tok| tok.value) {
            Some(TokenKind::Asterisk) | Some(TokenKind::Slash) => {
                let op = match tokens.next().unwrap() {
                    Token {
                        value: TokenKind::Asterisk,
                        loc,
                    } => BinOp::mult(loc),
                    Token {
                        value: TokenKind::Slash,
                        loc,
                    } => BinOp::div(loc),
                    _ => unreachable!(),
                };
                let r = parse_expr1(tokens)?;
                let loc = e.loc.merge(&r.loc);
                e = Ast::binop(op, e, r, loc)
            }
            _ => return Ok(e),
        }
    }
}
fn parse_left_binop<Tokens>(
    tokens: &mut Peekable<Tokens>,
    subexpr_parser: fn(&mut Peekable<Tokens>) -> Result<Ast, ParseError>,
    op_parser: fn(&mut Peekable<Tokens>) -> Result<BinOp, ParseError>,
) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
    let mut e = subexpr_parser(tokens)?;
    loop {
        match tokens.peek() {
            Some(_) => {
                let op = match op_parser(tokens) {
                    Ok(op) => op,
// ここでパースに失敗したのはこれ以上中置演算子がないという意味
                    Err(_) => break,
                };
                let r = subexpr_parser(tokens)?;
                let loc = e.loc.merge(&r.loc);
                e = Ast::binop(op, e, r, loc)
            }
            _ => break,
        }
    }
    Ok(e)
}
// expr1
fn parse_expr1<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
    match tokens.peek().map(|tok| tok.value) {
        Some(TokenKind::Plus) | Some(TokenKind::Minus) => {
// ("+" | "-")
            let op = match tokens.next() {
                Some(Token {
                         value: TokenKind::Plus,
                         loc,
                     }) => UniOp::plus(loc),
                Some(Token {
                         value: TokenKind::Minus,
                         loc,
                     }) => UniOp::minus(loc),
                _ => unreachable!(),
            };
// , ATOM
            let e = parse_atom(tokens)?;
            let loc = op.loc.merge(&e.loc);
            Ok(Ast::uniop(op, e, loc))
        }
// | ATOM
        _ => parse_atom(tokens),
    }
}
// atom
fn parse_atom<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Ast, ParseError>
    where
        Tokens: Iterator<Item = Token>,
{
    tokens
        .next()
        .ok_or(ParseError::Eof)
        .and_then(|tok| match tok.value {
// UNUMBER
            TokenKind::Number(n) => Ok(Ast::new(AstKind::Num(n), tok.loc)),
// | "(", EXPR3, ")" ;
            TokenKind::LParen => {
                let e = parse_expr(tokens)?;
                match tokens.next() {
                    Some(Token {
                             value: TokenKind::RParen,
                             ..
                         }) => Ok(e),
                    Some(t) => Err(ParseError::RedundantExpression(t)),
                    _ => Err(ParseError::UnclosedOpenParen(tok)),
                }
            }
            _ => Err(ParseError::NotExpression(tok)),
        })
}

use std::io;
/// プロンプトを表示しユーザの入力を促す
fn prompt(s: &str) -> io::Result<()> {
    use std::io::{stdout, Write};
    let stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write(s.as_bytes())?;
    stdout.flush()
}

fn main() {
    use std::io::{stdin, BufRead, BufReader};
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdin = BufReader::new(stdin);
    let mut lines = stdin.lines();
    loop {
        prompt("> ").unwrap();
        if let Some(Ok(line)) = lines.next() {
            let tokens = lex(&line).unwrap();
// 字句解析した結果をパースし
            let ast = parse(tokens).unwrap();
// 出力する
            println!("{:?}", ast);
        } else {
            break;
        }
    }
}

#[test]
fn test_parser() {
// 1 + 2 * 3 - -10
    let ast = parse(vec![
        Token::number(1, Loc(0, 1)),
        Token::plus(Loc(2, 3)),
        Token::number(2, Loc(4, 5)),
        Token::asterisk(Loc(6, 7)),
        Token::number(3, Loc(8, 9)),
        Token::minus(Loc(10, 11)),
        Token::minus(Loc(12, 13)),
        Token::number(10, Loc(13, 15)),
    ]);
    assert_eq!(
        ast,
        Ok(Ast::binop(
            BinOp::sub(Loc(10, 11)),
            Ast::binop(
                BinOp::add(Loc(2, 3)),
                Ast::num(1, Loc(0, 1)),
                Ast::binop(
                    BinOp::new(BinOpKind::Mult, Loc(6, 7)),
                    Ast::num(2, Loc(4, 5)),
                    Ast::num(3, Loc(8, 9)),
                    Loc(4, 9)
                ),
                Loc(0, 9),
            ),
            Ast::uniop(
                UniOp::minus(Loc(12, 13)),
                Ast::num(10, Loc(13, 15)),
                Loc(12, 15)
            ),
            Loc(0, 15)
        ))
    )
}