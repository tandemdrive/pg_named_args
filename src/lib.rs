#![doc = include_str!("../README.md")]

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse2, parse_macro_input, parse_quote,
    spanned::Spanned,
    token::Brace,
    ExprStruct, ItemStruct, LitStr, Member, Token,
};

/// This macro implements named parameters, for use with tokio_postgres.
///
/// The macro returns a tuple containing the query and the parameter slice that
/// can be used to call the various query methods provided by tokio_postgres.
///
/// A complete example could look something like:
/// ```ignore
/// let name = "Fred";
/// let surname = "Flintstone";
/// let (query, params) = query_args!(
///        r"INSERT INTO flintstone($[name, surname]) VALUES($[..])",
///        Args { name, surname }
///    );
/// txn.execute(query, params).await?;
/// ```
///
/// A number of different syntactical forms are supported see the [integration
/// tests](https://git.service.d11n.nl/dreamsolution/tandemdrive/src/branch/main/share/proc_macros/tests/integration/main.rs) for examples.
// #[proc_macro]
#[proc_macro]
pub fn query_args(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_raw = TokenStream::from(input.clone());
    let format = parse_macro_input!(input as Format);
    let mut errors = vec![];

    let mut names = vec![];
    let template = rewrite_query(format.template, &mut names, &mut errors);

    let def = struct_def(&format.args_name, &names);
    if (&format.args_name) != "Args" {
        errors.push(syn::Error::new_spanned(
            &format.args_name,
            "expected struct name to be `Args`",
        ));
    }

    let mut init = TokenStream::new();
    format.args_name.to_tokens(&mut init);
    format
        .args_brace
        .surround(&mut init, |init| format.args_inner.to_tokens(init));

    let params = if let Ok(res) = parse2::<ExprStruct>(init) {
        let params = names
            .iter()
            .filter_map(|search| {
                res.fields.iter().find_map(|field| {
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
                quote!(#res as &(dyn ::postgres_types::ToSql + Sync))
            });
        quote!(&[#(#params),*])
    } else {
        quote!(&[])
    };

    let errors = errors.into_iter().map(|err| err.to_compile_error());

    quote!({
        #(#errors;)*
        #[allow(unreachable_code)]
        if false {
            unreachable!();
            #def;
            (#input_raw);
        }
        (#template, #params)
    })
    .into()
}

fn struct_def(name: &Ident, names: &[String]) -> ItemStruct {
    let idents = names.iter().map(|x| Ident::new_raw(x, Span::call_site()));
    let generics = names
        .iter()
        .map(|x| Ident::new_raw(&format!("_{x}"), Span::call_site()));
    let generics2 = generics.clone();

    parse_quote!(struct #name <#(#generics),*> {
        #(#idents: #generics2,)*
    })
}

fn rewrite_query(inp: LitStr, names: &mut Vec<String>, errors: &mut Vec<syn::Error>) -> LitStr {
    let span = inp.span();
    let mut inp = &*inp.value();
    let mut template = String::new();
    let mut batch = "";

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

        let ident_len = inp.find(|x: char| !ident_char(x)).unwrap_or(inp.len());
        let ident = &inp[..ident_len];
        inp = &inp[ident_len..];

        if ident.is_empty() {
            let Some("[") = inp.get(..1) else {
                errors.push(syn::Error::new(span, "expected ident or [ after $"));
                return LitStr::new(&template, span);
            };
            inp = &inp[1..];

            let until = inp
                .find(|x: char| !ident_char(x) && !x.is_ascii_whitespace() && x != ',' && x != '.')
                .unwrap_or(inp.len());
            let columns = &inp[..until];
            inp = &inp[until..];

            let Some("]") = inp.get(..1) else {
                errors.push(syn::Error::new(span, "expected closing ]"));
                return LitStr::new(&template, span);
            };
            inp = &inp[1..];

            if columns == ".." {
                if batch.is_empty() {
                    errors.push(syn::Error::new(span, "$[..] is empty"));
                    continue;
                }

                let mut out = vec![];
                for column in batch.split(',') {
                    let ident = column.trim();
                    let idx = get_idx(ident);
                    out.push(format!("${}", idx + 1));
                }

                template.push_str(&out.join(", "));
                batch = "";
            } else {
                if !batch.is_empty() {
                    errors.push(syn::Error::new(span, "$[..] is not used"));
                }

                template.push_str(columns);
                batch = columns;
            }
        } else {
            let idx = get_idx(ident);
            template.push_str(&format!("${}", idx + 1));
        }
    }

    if !batch.is_empty() {
        errors.push(syn::Error::new(span, "$[..] is not used"));
    }

    LitStr::new(&template, span)
}

struct Format {
    template: LitStr,
    _comma: Token![,],
    args_name: Ident,
    args_brace: Brace,
    args_inner: proc_macro2::TokenStream,
}

impl Parse for Format {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let inner;
        Ok(Format {
            template: input.parse()?,
            _comma: input.parse()?,
            args_name: input.parse()?,
            args_brace: braced!(inner in input),
            args_inner: inner.parse()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite_query_wrapper(format: &str) -> Result<String, Vec<syn::Error>> {
        let mut errors = vec![];
        let mut names = vec![];
        let inp = LitStr::new(format, Span::call_site());
        let res = rewrite_query(inp, &mut names, &mut errors);
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
                "$[..] is empty",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three]
) VALUES (
    $[..], $[..]
);
                ",
                "$[..] is empty",
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
                "$[..] is not used",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three]
) VALUES (
    $one, $two, $three
);
                ",
                "$[..] is not used",
            ),
            (
                r"
INSERT INTO some_table (
    $[one, two, three
) VALUES (
    $[..]
);
                ",
                "expected closing ]",
            ),
            (
                r"
INSERT INTO some_table (
    $ one, two, three]
) VALUES (
    $[..]
);
                ",
                "expected ident or [ after $",
            ),
        ];

        for (format, err) in tests {
            let errors = rewrite_query_wrapper(format).unwrap_err();
            let error_msgs: Vec<_> = errors.into_iter().map(|x| x.to_string()).collect();
            assert_eq!(error_msgs.len(), 1, "{error_msgs:?}");
            assert_eq!(error_msgs[0], err);
        }
    }

    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");
    }
}
