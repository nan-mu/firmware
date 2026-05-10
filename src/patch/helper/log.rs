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

// 1. 尝试匹配 %d 等。成功会消耗相应字节。
fn parse_spec_type(input: &[u8]) -> IResult<&[u8], SpecType> {
    alt((
        // %pI4 - IPv4 地址格式 (4 字节)
        value(SpecType::Dot, tag(b"%pI4" as &[u8])),
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
                error!("[helper]bpf_printk: buffer full, skip text.");
                info!("Consider modifying the configuration file settings regarding the cache area.")
            },
            Token::Spec(spec_type) => {
                if let Some(&val) = arg.next() {
                    let result = match spec_type {
                        SpecType::Signed => write!(buf, "{}", val as isize),
                        SpecType::Unsigned => write!(buf, "{}", val),
                        SpecType::Hex => write!(buf, "{:#x}", val),
                        SpecType::Octal => write!(buf, "{:#o}", val),
                        SpecType::String => {
                            // val 是字符串指针
                            unsafe {
                                // 假设最大字符串长度 128
                                let ptr = val as *const u8;
                                let mut len = 0;
                                while len < 128 && ptr.add(len).read() != 0 {
                                    len += 1;
                                }
                                let slice = core::slice::from_raw_parts(ptr, len);
                                match core::str::from_utf8(slice) {
                                    Ok(s) => write!(buf, "{}", s),
                                    Err(_) => write!(buf, "<invalid utf8>"),
                                }
                            }
                        },
                        SpecType::Dot => {
                            // %pI4 - IPv4 地址 (val 是指向 4 字节的指针)
                            unsafe {
                                let ptr = val as *const u8;
                                let ip = [ptr.read(), ptr.add(1).read(), ptr.add(2).read(), ptr.add(3).read()];
                                write!(buf, "{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
                            }
                        },
                    };
                    
                    if result.is_err() {
                        error!("[helper]bpf_printk: buffer full, skip value.");
                        info!("Consider modifying the configuration file settings regarding the cache area.")
                    }
                } else {
                    error!("[helper]bpf_printk: not enough arguments for format specifier");
                }
            }
        }
    }
    buf
}
