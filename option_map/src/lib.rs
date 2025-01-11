use proc_macro::{Delimiter, TokenStream, TokenTree};
// Necessary for the `TokenStream::from_str()` implementation
use std::str::FromStr;

enum OptArg {
    None,
    Single,
    Multi,
}
enum NameCase {
    Lower,
    Sentence,
}

const MISSING_OPT_ARG_IDENT_MSG: &'static str =
  "Missing an identifier for the option-argument. Currently, this is limited to `String`";

fn to_case(ident: &str, case: &NameCase) -> String {
    let mut out = String::with_capacity(ident.len());
    match case {
        NameCase::Lower => {
            let mut chars = ident.chars().enumerate();
            let mut start = 0;
            while let Some((i, c)) = chars.next() {
                if c.is_uppercase() {
                    out.push_str(&ident[start..i]);
                    if i > 0 {
                        out.push('-');
                    }
                    out.extend(c.to_lowercase());
                    start = i + c.len_utf8();
                }
            }
            out.push_str(&ident[start..]);
        }
        NameCase::Sentence => {
            let mut chars = ident.chars();
            // Unwrap safe when called in parsing logic because
            // this function only called when there's a value
            out.extend(chars.next().unwrap().to_uppercase());
            while let Some(c) = chars.next() {
                if c.is_uppercase() {
                    out.push('-');
                    out.extend(c.to_lowercase());
                    continue;
                }
                out.push(c);
            }
        }
    }
    out
}

/// Define the options available to the program.
/// The output is an enum that can be matched on.
///
/// If constructing the `Router` manually, not using
/// `router!()`, the generated enum has a `list()`
/// method to supply option data.
///
/// # Example
///
/// ```ignore
/// optmap!(enum Name using [
///   // A variant with no extra properties
///   Variant1,
///   // A variant with a short alias
///   Variant2 | 'a',
///   // A shorthand-only variant
///   'Q'
///   // A variant that expects one argument (Must be `String`)
///   Variant3 > String,
///   // A variant that expects one or more arguments (Must be `String`)
///   Variant5 > String[],
///   // A variant with a short alias and an argument
///   Variant4 | 'b' > String,
///   /// This doc comment will become the option's summary text
///   Variant5,
///   /// Can change the case of the name: Sentence
///   #[case="sentence"]
///   SentenceCase,
/// ]);
/// ```
#[proc_macro]
pub fn optmap(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();
    // Decision:
    // Work with strings instead of token streams since it's
    // faster. I guess it's because the string form is much
    // more compact, being much less code to compile
    let mut out = String::with_capacity(42);
    // (option name, summary, type, the option's attribute tokens, shorthand, opt arg kind)
    let mut opt_variants = Vec::<(
        String,
        String,
        String,
        String,
        Option<char>,
        OptArg,
        NameCase,
    )>::new();

    out.push_str("#[repr(u16)]#[derive(Clone,Copy)]");

    if input
        .next()
        .and_then(|mut t| {
            let mut s = t.to_string();
            out.push_str(&t.to_string());
            if s == "pub" {
                t = input.next()?;
                s = t.to_string();
                out.push_str(" enum");
            }
            if s == "enum" {
                return Some(());
            }
            None
        })
        .is_none()
    {
        panic!("Begin by declaring that this is an enum, i.e. `enum <enum Name>`");
    }

    let name_tok =
        input.next().expect("Missing an identifier for the enum.");
    let name = name_tok.to_string();
    out.push(' ');
    out.push_str(&name);

    if input.next().is_some_and(|t| t.to_string() == "using") {
    } else {
        panic!("Missing keyword `using`");
    }

    let mut input = match input.next() {
        Some(TokenTree::Group(g))
            if g.delimiter() == Delimiter::Bracket =>
        {
            g.stream()
        }
        _ => panic!("Options are defined inside of `[]`"),
    }
    .into_iter();

    let mut doc = String::new();
    let mut doc_bytes_to_trim = None;
    let mut help_opt_pos = None;
    let mut wordbreak = false;

    // Add one so we don't have to check in the loop
    opt_variants.push((
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        None,
        OptArg::None,
        NameCase::Lower,
    ));
    let mut variant = 0;
    while let Some(t) = input.next() {
        // Looking for variant, or attribute
        match t {
            TokenTree::Punct(ref p) if *p == '#' => {
                match input.next().unwrap() {
                    TokenTree::Group(attr) => {
                        let mut attr_tokens = attr.stream().into_iter();
                        match attr_tokens.next() {
                            Some(tok) if tok.to_string() == "doc" => {
                                // Skip the '=' token
                                attr_tokens.next();
                                let text = attr_tokens
                                    .next()
                                    .unwrap()
                                    .to_string();
                                // Remove quotes
                                let text = &text[1..text.len() - 1];
                                let count = match doc_bytes_to_trim {
                                    Some(count) => count,
                                    _ => {
                                        let count = text
                                            .find(|c| c != ' ')
                                            .unwrap_or(0);
                                        doc_bytes_to_trim = Some(count);
                                        count
                                    }
                                };
                                if text.is_empty() {
                                    // Newlines would otherwise be lost
                                    wordbreak = true;
                                } else {
                                    if wordbreak {
                                        doc.push_str("\n\n");
                                        // Don't include spaces after a break
                                        doc.push_str(&text[count..]);
                                    } else if doc.is_empty() {
                                        doc.push_str(&text[count..])
                                    } else {
                                        doc.push_str(text);
                                    }
                                }
                            }
                            Some(tok) if tok.to_string() == "case" => {
                                // Skip the '=' token
                                attr_tokens.next();
                                let text = attr_tokens
                                    .next()
                                    .unwrap()
                                    .to_string();
                                // Remove quotes
                                let text = text[1..text.len() - 1].trim();
                                if !text.is_empty() {
                                    opt_variants[variant].6 = match text {
                                        "lower" => NameCase::Lower,
                                        "sentence" => NameCase::Sentence,
                                        _ => {
                                            panic!("Invalid value for `case` attribute");
                                        }
                                    };
                                }
                            }
                            Some(_) => {
                                opt_variants[variant].3.push('#');
                                opt_variants[variant]
                                    .3
                                    .push_str(&attr.to_string());
                            }
                            _ => panic!("Unexpected token."),
                        }
                    }
                    tok => {
                        opt_variants[variant].3.push('#');
                        opt_variants[variant].3.push_str(&tok.to_string());
                    }
                }
                continue;
            }
            TokenTree::Ident(ident) => {
                opt_variants[variant].0 = ident.to_string();
                if opt_variants[variant].0.to_lowercase() == "help" {
                    help_opt_pos = Some(variant);
                }
                if !doc.is_empty() {
                    opt_variants[variant].3.push_str("#[doc=\"");
                    opt_variants[variant].3.push_str(&doc);
                    opt_variants[variant].3.push_str("\"]");
                    opt_variants[variant].1 = doc.to_owned();
                    doc.clear();
                }
            }
            TokenTree::Literal(lit) => {
                let chars = lit.to_string();
                let mut chars = chars.chars();
                if chars.next().unwrap() != '\'' {
                    panic!("Expected a `char`");
                }
                let c = chars.next().unwrap();

                // TODO: Enable through a Posix feature only
                if !c.is_alphanumeric() {
                    panic!("An Option's short alias must be an alphanumeric character from the portable character set.");
                }
                opt_variants[variant].4 = Some(c);

                opt_variants[variant].0 = c.to_string();
                if !doc.is_empty() {
                    opt_variants[variant].3.push_str("#[doc=\"");
                    opt_variants[variant].3.push_str(&doc);
                    opt_variants[variant].3.push_str("\"]");
                    opt_variants[variant].1 = doc.to_owned();
                    doc.clear();
                }
            }
            _ => {}
        }
        // Looking for option shorthand or type
        let t2 = input.next();
        match t2 {
            Some(TokenTree::Punct(p)) => match p.as_char() {
                ',' => {
                    variant += 1;
                    opt_variants.push((
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                        None,
                        OptArg::None,
                        NameCase::Lower,
                    ));
                    continue;
                }
                '|' => {
                    if opt_variants[variant].4.is_some() {
                        panic!("Already gave an option shorthand")
                    }
                    // Add short to variant
                    match input
                        .next()
                        .expect("Missing short alias. If this was intentional, remove the '|'.")
                    {
                        TokenTree::Literal(lit) => {
                            let chars = lit.to_string();
                            let mut chars = chars.chars();
                            if chars.next().unwrap() != '\'' {
                                panic!("Expected a `char`");
                            }
                            let c = chars.next().unwrap();

                            // TODO: Enable through a Posix feature only
                            if !c.is_alphanumeric() {
                                panic!("An Option's short alias must be an alphanumeric character from the portable character set.");
                            }
                            opt_variants[variant].4 = Some(c);
                        }
                        _ => panic!("Expected a `char`."),
                    };
                }
                '>' => {
                    // Add value to variant
                    opt_variants[variant].2 = match input.next() {
                        Some(TokenTree::Ident(v)) => v.to_string(),
                        _ => panic!("{}", MISSING_OPT_ARG_IDENT_MSG),
                    };
                    match input.next() {
                        Some(TokenTree::Group(g))
                            if g.delimiter() == Delimiter::Bracket
                                && g.stream().is_empty() =>
                        {
                            opt_variants[variant].5 = OptArg::Multi;
                            match input.next() {
                                Some(TokenTree::Punct(p))
                                    if p.as_char() == ',' =>
                                {
                                    variant += 1;
                                    opt_variants.push((
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                        None,
                                        OptArg::None,
                                        NameCase::Lower,
                                    ));
                                    continue;
                                }
                                None => {
                                    // Skip over the brackets
                                    input.next().unwrap();
                                    break;
                                }
                                Some(tok) => {
                                    panic!("Unexpected token {}", tok)
                                }
                            }
                        }
                        Some(TokenTree::Punct(p))
                            if p.as_char() == ',' =>
                        {
                            opt_variants[variant].5 = OptArg::Single;
                            variant += 1;
                            opt_variants.push((
                                String::new(),
                                String::new(),
                                String::new(),
                                String::new(),
                                None,
                                OptArg::None,
                                NameCase::Lower,
                            ));
                            continue;
                        }
                        None => {
                            opt_variants[variant].5 = OptArg::Single;
                            // No more tokens
                            break;
                        }
                        _ => panic!("Unexpected token."),
                    }
                }
                t => panic!("Unexpected token {}", t),
            },
            Some(t) => panic!("Unexpected token '{}'", t),
            _ => break,
        }
        // Found option shorthand, now looking for arg or ','
        match input.next() {
            Some(TokenTree::Punct(p)) => {
                match p.as_char() {
                    ',' => {
                        variant += 1;
                        opt_variants.push((
                            String::new(),
                            String::new(),
                            String::new(),
                            String::new(),
                            None,
                            OptArg::None,
                            NameCase::Lower,
                        ));
                    }
                    '>' => {
                        // Add value to variant
                        opt_variants[variant].2 = match input.next() {
                            Some(TokenTree::Ident(v)) => v.to_string(),
                            _ => panic!("{}", MISSING_OPT_ARG_IDENT_MSG),
                        };
                        match input.next() {
                            Some(TokenTree::Group(g))
                                if g.delimiter() == Delimiter::Bracket
                                    && g.stream().is_empty() =>
                            {
                                opt_variants[variant].5 = OptArg::Multi;
                                match input.next() {
                                    Some(TokenTree::Punct(p))
                                        if p.as_char() == ',' => {}
                                    None => {
                                        // Skip over the brackets
                                        break;
                                    }
                                    Some(tok) => {
                                        panic!("Unexpected token {}", tok)
                                    }
                                }
                            }
                            Some(TokenTree::Punct(p))
                                if p.as_char() == ',' =>
                            {
                                opt_variants[variant].5 = OptArg::Single;
                            }
                            None => {
                                opt_variants[variant].5 = OptArg::Single;
                                // No more tokens
                                break;
                            }
                            _ => panic!("Unexpected token."),
                        }
                        variant += 1;
                        opt_variants.push((
                            String::new(),
                            String::new(),
                            String::new(),
                            String::new(),
                            None,
                            OptArg::None,
                            NameCase::Lower,
                        ));
                        continue;
                    }
                    _ => {}
                }
            }
            None => {}
            _ => {}
        }
    }

    // Always has one extra
    opt_variants.pop();

    let mut shorthands = String::new();
    let mut router_opts = String::new();
    let mut summaries = String::new();
    let mut names = String::new();
    let mut enum_variants = String::new();
    variant = 0;

    // For binary search at runtime
    opt_variants
        .sort_unstable_by(|(a, ..), (b, ..)| a.partial_cmp(b).unwrap());

    for o in opt_variants.into_iter() {
        if let Some(c) = o.4 {
            shorthands.push('(');
            shorthands.push_str(&variant.to_string());
            shorthands.push_str(",\'");
            shorthands.push(c);
            shorthands.push_str("\'),");
        }
        enum_variants.push_str(&o.3.to_string());
        enum_variants.push_str(&o.0);
        enum_variants.push(',');

        router_opts.push_str("router::Opt{name:");
        router_opts.push_str(&variant.to_string());
        router_opts.push_str(",kind:router::OptArgKind::");
        router_opts.push_str(match o.5 {
            OptArg::Multi => "Multiple},",
            OptArg::None => "KeyOnly},",
            OptArg::Single => "Single},",
        });

        summaries.push_str("\"");
        summaries.push_str(&o.1);
        summaries.push_str("\",");

        names.push_str("\"");
        names.push_str(&to_case(&o.0, &o.6));
        names.push_str("\",");
        variant += 1;
    }

    out.push('{');
    out.push_str(&enum_variants);
    out.push_str("}impl Into<usize> for ");
    out.push_str(&name);
    out.push_str(
        "{fn into(self)->usize{self as usize}}impl Into<u16> for ",
    );
    out.push_str(&name);
    out.push_str("{fn into(self)->u16{self as u16}}impl ");
    out.push_str(&name);
    out.push_str(
        "{#[inline(always)]
      #[doc=\"Call this function as an argument to `router::run()`\"]
      pub const fn list()->(
        &'static[router::Opt],
        &'static[(u16,char)],
        &'static[&'static str],
        &'static[&'static str],
        Option<u16>){(
          &[",
    );
    out.push_str(&router_opts);
    out.push_str("],&[");
    out.push_str(&shorthands);
    out.push_str("],&[");
    out.push_str(&names);
    out.push_str("],&[");
    out.push_str(&summaries);
    out.push_str("],");
    match help_opt_pos {
        Some(pos) => {
            out.push_str("Some(");
            out.push_str(&pos.to_string());
            out.push(')');
        }
        _ => out.push_str("None"),
    }
    out.push_str(")}}");

    // println!("{}", out);
    // out
    TokenStream::from_str(&out).unwrap()
    // TokenStream::from_str("enum O {}").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_an_ident_to_kebab_case() {
        assert_eq!(
            to_case("String", &NameCase::Lower),
            "string".to_string()
        );
        assert_eq!(
            to_case("TwoWords", &NameCase::Lower),
            "two-words".to_string()
        );
        assert_eq!(
            to_case("SomeManyWordsHere", &NameCase::Lower),
            "some-many-words-here".to_string()
        );
    }
}
