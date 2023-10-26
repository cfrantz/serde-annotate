use syn::parse::ParseStream;
use syn::{parenthesized, Attribute, Error, Ident, LitStr, Result, Token};

#[derive(Debug, PartialEq)]
pub enum Format {
    None,
    Block,
    Binary,
    Decimal,
    Hex,
    Octal,
    Compact,
    HexStr,
    Hexdump,
    Xxd,
}

#[derive(Debug, PartialEq)]
pub enum Comment {
    None,
    Field(Ident),
    Function(Ident),
    Static(String),
}

#[derive(Debug)]
pub struct Attrs<'a> {
    pub annotate: Option<&'a Attribute>,
    pub format: Format,
    pub comment: Comment,
}

pub fn get(input: &[Attribute]) -> Result<Attrs> {
    let mut attrs = Attrs {
        annotate: None,
        format: Format::None,
        comment: Comment::None,
    };

    for attr in input {
        if attr.path().is_ident("annotate") {
            attrs.annotate = Some(attr);
            parse_annotate_attribute(&mut attrs, attr)?;
        }
    }
    Ok(attrs)
}

fn function_call(input: ParseStream) -> Result<bool> {
    let content;
    let _result = parenthesized!(content in input);
    Ok(content.is_empty())
}

fn parse_annotate_attribute<'a>(attrs: &mut Attrs<'a>, attr: &'a Attribute) -> Result<()> {
    syn::custom_keyword!(format);
    syn::custom_keyword!(comment);

    attr.parse_args_with(|input: ParseStream| {
        let mut more = true;
        while more {
            if input.peek(format) {
                let _kw = input.parse::<format>()?;
                let _eq: Token![=] = input.parse()?;
                let ident: Ident = input.parse()?;
                let istr = ident.to_string();
                let format = match istr.as_str() {
                    "block" => Format::Block,
                    "bin" => Format::Binary,
                    "dec" => Format::Decimal,
                    "oct" => Format::Octal,
                    "hex" => Format::Hex,
                    "hexstr" => Format::HexStr,
                    "hexdump" => Format::Hexdump,
                    "xxd" => Format::Xxd,
                    "compact" => Format::Compact,
                    _ => Format::None,
                };
                if format == Format::None {
                    return Err(Error::new_spanned(attr, "unknown annotation type"));
                }
                attrs.format = format;
            } else if input.peek(comment) {
                let _kw = input.parse::<comment>()?;
                let _eq: Token![=] = input.parse()?;
                if input.peek(Ident) {
                    let ident: Ident = input.parse()?;
                    let func = function_call(input);
                    attrs.comment = match func {
                        Ok(true) => Comment::Function(ident.clone()),
                        Ok(false) => {
                            return Err(Error::new_spanned(attr, "Function args not permitted"));
                        }
                        Err(_) => Comment::Field(ident.clone()),
                    };
                } else {
                    let comment: LitStr = input.parse()?;
                    attrs.comment = Comment::Static(comment.value());
                }
            } else {
                return Err(Error::new_spanned(attr, "parse error"));
            }

            more = input.peek(Token![,]);
            if more {
                let _comma: Token![,] = input.parse()?;
                more = !input.is_empty();
            }
        }
        Ok(())
    })
}
