use core::fmt::Write;
use heapless::{String, Vec};
use log::{error, info};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, rest, value, verify},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum SpecType {
    Signed,
    Unsigned,
    String,
    Hex,
    Octal,
    Dot,
}

impl Default for SpecType {
    fn default() -> Self {
        Self::Signed
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum Token<'a> {
    Text(&'a [u8]),
    Spec(SpecType),
}

// 1. 尝试匹配 %d 等。成功会消耗 2 字节。
fn parse_spec_type(input: &[u8]) -> IResult<&[u8], SpecType> {
    alt((
        value(
            SpecType::Signed,
            alt((tag(b"%d" as &[u8]), tag(b"%i" as &[u8]))),
        ),
        value(SpecType::Unsigned, tag(b"%u" as &[u8])),
        value(SpecType::String, tag(b"%s" as &[u8])),
        value(
            SpecType::Hex,
            alt((tag(b"%x" as &[u8]), tag(b"%X" as &[u8]))),
        ),
        value(SpecType::Octal, tag(b"%o" as &[u8])),
        value(SpecType::Dot, tag(b"%." as &[u8])),
    ))
    .parse(input)
}

// 2. 匹配直到 %，但必须至少消耗 1 字节 (verify 确保不返回空 Text)
fn parse_text_token<'a>(input: &'a [u8]) -> IResult<&'a [u8], Token<'a>> {
    verify(map(take_until(b"%" as &[u8]), Token::Text), |t| {
        if let Token::Text(s) = t {
            !s.is_empty()
        } else {
            false
        }
    })
    .parse(input)
}

// 3. 如果当前是 % 但不是合法的 Spec，消耗这个 % 存为文本
fn parse_single_percent<'a>(input: &'a [u8]) -> IResult<&'a [u8], Token<'a>> {
    map(tag(b"%" as &[u8]), |_| Token::Text(b"%")).parse(input)
}

// 4. 主解析器
pub fn parse_bpf_format<'a>(input: &'a [u8]) -> Vec<Token<'a>, 16> {
    let mut results = Vec::<Token, 16>::new();
    let mut curr_input = input;

    // 手动循环解析，直到出错或输入耗尽
    while !curr_input.is_empty() {
        let (next_input, token) = match alt((
            map(parse_spec_type, Token::Spec),
            parse_text_token,
            parse_single_percent,
            map(verify(rest, |r: &[u8]| !r.is_empty()), Token::Text),
        ))
        .parse(curr_input)
        {
            Ok(a) => a,
            Err(_e) => {
                error!("[helper]bpf_printk: parse error");
                break;
            }
        };

        // 尝试推入 heapless::Vec，如果满了就报错或停止
        if results.push(token).is_err() {
            // 这里可以根据需求选择报错还是直接返回当前已解析的
            break;
        }
        curr_input = next_input;
    }

    results
}

pub fn zip_format(tokens: Vec<Token, 16>, args: &[usize]) -> String<256> {
    // 根据需求调整容量
    let mut buf: String<256> = String::new();
    let mut arg = args.iter();
    for token in tokens {
        match token {
            Token::Text(s) => if write!(buf, "{}", match core::str::from_utf8(s) {
                Ok(s) => s,
                Err(e) => {
                    error!("[helper]bpf_printk, patch::helper::log::zip_format's core::str::from_utf8 failed. Check ebpf code.");
                    error!("Inner error: {:?}", e);
                    ""
                }
            }
            ).is_err() {
                error!("[helper]bpf_printk: buffer full, skill text.");
                info!("Consider modifying the configuration file settings regarding the cache area.")
            },
            Token::Spec(s) => match s {
                SpecType::Signed => if write!(buf, "{}", arg.next().unwrap()).is_err() {
                    error!("[helper]bpf_printk: buffer full, skill value.");
                    info!("Consider modifying the configuration file settings regarding the cache area.")
                }
                _ => panic!()
            }
        }
    }
    buf
}
