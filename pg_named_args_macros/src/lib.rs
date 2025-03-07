use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse2, parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    ExprStruct, ItemStruct, LitStr, Member, Token,
};

/// The macro returns a tuple containing the query and the parameter slice that
/// can be used to call the various query methods provided by rust_postgres/tokio_postgres.
///
/// Please refer to the crate level documentation for an overview of the syntax.
///
/// A complete example could look something like:
/// ```
/// # use pg_named_args::query_args;
/// let name = "Fred";
/// let surname = "Flintstone";
/// let (query, params) = query_args!(
///     r"INSERT INTO flintstone($[name, surname]) VALUES($[..])",
///     Args { name, surname }
/// );
/// ```
/// ```ignore
/// txn.execute(query, params).await?;
/// ```
#[proc_macro]
pub fn query_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_raw = TokenStream::from(input.clone());
    let format = parse_macro_input!(input as Format);
    let mut errors = vec![];

    let mut names = vec![];
    let mut fragments = vec![];
    let template = rewrite_query(format.template, &mut names, &mut errors, &mut fragments);

    let mut args = HashMap::new();
    format
        .args
        .into_iter()
        .flat_map(|x| x.1.into_iter())
        .for_each(|x| {
            // TODO: simplify this
            let mut init = TokenStream::new();
            x.name.to_tokens(&mut init);
            x.brace.surround(&mut init, |init| x.inner.to_tokens(init));
            // we only care when the struct parses, because we output the raw input which would otherwise give an error.
            let fields: Vec<_> = parse2::<ExprStruct>(init)
                .ok()
                .map(|inner| {
                    if let Some(dots) = inner.dot2_token {
                        errors.push(syn::Error::new_spanned(
                            dots,
                            "struct update syntax is not supported by the query_args macro",
                        ))
                    }

                    inner.fields.into_iter().collect()
                })
                .unwrap_or_default();

            // something is always inserted here as a proof that rustc will check the struct fields.
            if args.insert(x.name.to_string(), fields).is_some() {
                errors.push(syn::Error::new_spanned(x.name, "duplicate struct name"));
            }
        });

    let params: Vec<_> = args
        .remove("Args")
        .map(|fields| {
            // this will only be a list of the fields that actually exist.
            // if not all fields are specified it is a struct init error.
            names
                .iter()
                .filter_map(|search| {
                    fields.iter().find_map(|field| {
                        let Member::Named(name) = &field.member else {
                            return None;
                        };
                        (name.unraw() == *search).then_some(field.expr.clone())
                    })
                })
                .map(|res| {
                    // Make a reference using res.span() so that ToSql errors are shown nicely.
                    let res = quote_spanned!(res.span()=> &#res);
                    // Cast to &dyn without span to hide unnecessary cast warning
                    quote!(#res as &(dyn ::pg_named_args::postgres_types::ToSql + Sync))
                })
                .collect()
        })
        .unwrap_or_else(|| {
            if !names.is_empty() {
                errors.push(syn::Error::new(Span::call_site(), "expected `Args` struct"));
            }
            vec![]
        });

    let mut template = quote!(#template);
    let fragment_args: Vec<_> = args
        .remove("Sql")
        .map(|fields| {
            fragments
                .iter()
                .filter_map(|search| {
                    fields.iter().find_map(|field| {
                        let Member::Named(name) = &field.member else {
                            return None;
                        };
                        (name.unraw() == *search).then_some(field.expr.clone())
                    })
                })
                .map(|res| quote_spanned!(res.span()=> ::pg_named_args::Fragment::get(#res)))
                .collect()
        })
        .unwrap_or_else(|| {
            if !fragments.is_empty() {
                errors.push(syn::Error::new(Span::call_site(), "expected `Sql` struct"));
            }
            vec![]
        });

    // prevent additional errors when the Sql struct is not complete yet
    if fragment_args.len() == fragments.len() {
        template = quote!(&::std::format!(#template #(,#fragment_args)*));
    }

    for key in args.keys() {
        errors.push(syn::Error::new(
            Span::call_site(),
            format!("unknown struct name `{key}`"),
        ));
    }

    let def = struct_def(&names);
    let def2 = struct_def2(&fragments);
    let errors = errors.into_iter().map(|err| err.to_compile_error());

    quote!({
        #(#errors;)*
        #[allow(
            unreachable_code,
            clippy::empty_structs_with_brackets,
            clippy::inconsistent_struct_constructor,
            clippy::literal_string_with_formatting_args,
        )]
        if false {
            unreachable!();
            #def;
            #def2;
            (#input_raw);
        }
        (#template, &[#(#params),*])
    })
    .into()
}

fn struct_def(names: &[String]) -> ItemStruct {
    let idents = names.iter().map(|x| Ident::new_raw(x, Span::call_site()));
    let generics = names
        .iter()
        .map(|x| Ident::new_raw(&format!("_{x}"), Span::call_site()));
    let generics2 = generics.clone();

    parse_quote!(struct Args<#(#generics),*> {
        #(#idents: #generics2,)*
    })
}

fn struct_def2(fragments: &[String]) -> ItemStruct {
    let fragment_idents = fragments
        .iter()
        .map(|x| Ident::new_raw(x, Span::call_site()));

    parse_quote!(struct Sql {
        #(#fragment_idents: ::pg_named_args::Fragment,)*
    })
}

fn rewrite_query(
    inp: LitStr,
    names: &mut Vec<String>,
    errors: &mut Vec<syn::Error>,
    fragments: &mut Vec<String>,
) -> LitStr {
    let span = inp.span();
    let mut push_err = |message: &str| errors.push(syn::Error::new(span, message));

    let mut inp = &*inp.value().replace("{", "{{").replace("}", "}}");

    let mut template = String::new();
    let mut batch = None::<String>;

    let mut get_idx = |ident: &str| {
        if let Some(idx) = names.iter().position(|x| x == ident) {
            idx
        } else {
            names.push(ident.to_owned());
            names.len() - 1
        }
    };

    fn ident_char(x: char) -> bool {
        x.is_alphanumeric() || x == '_'
    }

    loop {
        let Some(dollar_pos) = inp.find('$') else {
            template.push_str(inp);
            break;
        };

        template.push_str(&inp[..dollar_pos]);
        inp = &inp[dollar_pos + 1..];

        let mut is_fragment = false;
        // braces have been pre-escaped
        if inp.get(..2) == Some("{{") {
            is_fragment = true;
            inp = &inp[2..];
        }

        let ident_len = inp.find(|x: char| !ident_char(x)).unwrap_or(inp.len());
        let ident = &inp[..ident_len];
        inp = &inp[ident_len..];

        if ident.is_empty() {
            if is_fragment {
                push_err("expected an identifer after `{`");
                return LitStr::new(&template, span);
            }

            let Some("[") = inp.get(..1) else {
                push_err("expected identifier or `[` after `$`");
                return LitStr::new(&template, span);
            };
            inp = &inp[1..];

            let until = inp
                .find(|x: char| !ident_char(x) && !x.is_ascii_whitespace() && x != ',' && x != '.')
                .unwrap_or(inp.len());
            let columns = &inp[..until];
            inp = &inp[until..];

            let Some("]") = inp.get(..1) else {
                push_err("expected closing `]`");
                return LitStr::new(&template, span);
            };
            inp = &inp[1..];

            if columns == ".." {
                let Some(columns) = batch.take() else {
                    push_err("parameter group is used, but not defined");
                    continue;
                };

                template.push_str(&columns);
            } else {
                let mut out = vec![];
                for column in columns.split(',') {
                    let ident = column.trim();
                    if ident.is_empty() {
                        push_err(
                            "expected identifier between all of `$[`, every `,` and final `]`",
                        );
                        continue;
                    }

                    let idx = get_idx(ident);
                    out.push(format!("${}", idx + 1));
                }

                if batch.replace(out.join(", ")).is_some() {
                    push_err("previous parameter group is not used");
                }

                template.push_str(columns);
            }
        } else if is_fragment {
            // braces have been pre-escaped
            if inp.get(..2) == Some("}}") {
                inp = &inp[2..];
            } else {
                push_err("fragment should end with `}`")
            }
            fragments.push(ident.to_owned());
            template.push_str("{}");
        } else {
            let idx = get_idx(ident);
            template.push_str(&format!("${}", idx + 1));
        }
    }

    if batch.is_some() {
        push_err("last parameter group is not used");
    }

    LitStr::new(&template, span)
}

struct RawStruct {
    name: Ident,
    brace: Brace,
    inner: proc_macro2::TokenStream,
}

impl Parse for RawStruct {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let inner;
        Ok(RawStruct {
            name: input.parse()?,
            brace: braced!(inner in input),
            inner: inner.parse()?,
        })
    }
}

struct Format {
    template: LitStr,
    args: Option<(Token![,], Punctuated<RawStruct, Token![,]>)>,
}

impl Parse for Format {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Format {
            template: input.parse()?,
            args: input
                .parse::<Option<Token![,]>>()?
                .map(|comma| {
                    let rest = input.parse_terminated(RawStruct::parse, Token![,])?;
                    syn::Result::Ok((comma, rest))
                })
                .transpose()?,
        })
    }
}

/// This macro creates a `Fragment` from a string literal.
///
/// Checking that the input is a string literal prevents accidental SQL injection with dynamic strings.
/// The resulting `Fragment` can be used with the [query_args] macro.
///
/// This is useful for creating dynamic queries where fragments can be swapped out for each other.
#[proc_macro]
pub fn fragment(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_raw = TokenStream::from(input.clone());

    let lit = parse_macro_input!(input as LitStr);
    let mut errors = None;
    let inp = lit.value();
    if inp.contains('$') {
        errors = Some(
            syn::Error::new(lit.span(), "Fragment is not allowed to contain `$`")
                .into_compile_error(),
        );
    }
    let res = quote!({
        #errors
        ::pg_named_args::Fragment::new_unchecked(#input_raw)
    });
    res.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite_query_wrapper(format: &str) -> Result<String, Vec<syn::Error>> {
        let mut errors = vec![];
        let mut names = vec![];
        let mut fragments = vec![];
        let inp = LitStr::new(format, Span::call_site());
        let res = rewrite_query(inp, &mut names, &mut errors, &mut fragments);
        if errors.is_empty() {
            Ok(res.value())
        } else {
            Err(errors)
        }
    }

    #[test]
    fn rewrite_query_impl_should_support_list_syntax() {
        let tests = [
            r"
INSERT INTO fred_flintstone(a, $[b, c])
VALUES(true, $[..]);
            ",
            r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $b, $c);
            ",
        ];

        for format in tests {
            let actual = rewrite_query_wrapper(format.trim()).unwrap();
            let expected = r"
INSERT INTO fred_flintstone(a, b, c)
VALUES(true, $1, $2);
                ";
            assert_eq!(actual, expected.trim());
        }
    }

    #[test]
    fn rewrite_query_should_error_on_unused() {
        let tests = [
            (
                r"
INSERT INTO some_table (
    one, two, three
) VALUES (
    $one, $two, $three, $[..]
);
                ",
                "parameter group is used, but not defined",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three]
) VALUES (
    $[..], $[..]
);
                ",
                "parameter group is used, but not defined",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three],
    $[one, two, three]
) VALUES (
    $[..]
);
                ",
                "previous parameter group is not used",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three]
) VALUES (
    $one, $two, $three
);
                ",
                "last parameter group is not used",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three
) VALUES (
    $[..]
);
                ",
                "expected closing `]`",
            ),
            (
                r"
INSERT INTO some_table (
    $ one, two, three]
) VALUES (
    $[..]
);
                ",
                "expected identifier or `[` after `$`",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two,]
) VALUES (
    $[..]
);
                ",
                "expected identifier between all of `$[`, every `,` and final `]`",
            ),
        ];

        for (format, err) in tests {
            let errors = rewrite_query_wrapper(format).unwrap_err();
            let error_msgs: Vec<_> = errors.into_iter().map(|x| x.to_string()).collect();
            assert_eq!(error_msgs.len(), 1, "{error_msgs:?}");
            assert_eq!(error_msgs[0], err);
        }
    }
}
