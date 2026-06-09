use self::RenameRule::*;

use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Field, Type, parse_macro_input};

use crate::parser::RenameRule::{Lowercase, ScreamingKebabCase, ScreamingSnakeCase};

pub(crate) fn derive_oocli_impl(input: TokenStream) -> TokenStream {
    // Parse input token stream as `DeriveInput`
    let input = parse_macro_input!(input as DeriveInput);

    let mut errors: Option<Error> = None;

    let DeriveInput { data, ident, .. } = input.clone();

    if let Data::Struct(data_struct) = data {
        let struct_args = match StructAttributes::from_derive_input(&input) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(e.write_errors());
            }
        };

        // initialize the base commands
        // todo: is there any risk to initializing it this way instead of separate
        // attributes per command/sub?
        let cmd_str = &struct_args.command;
        let mut output = quote!();
        for (idx, it) in cmd_str.split_whitespace().enumerate() {
            // initialize command with binary
            if idx == 0 {
                output.extend(quote! {
                    let mut cmd = std::process::Command::new(#it);
                });
            }
            // add any subcommands
            else {
                output.extend(quote! {
                    cmd.arg(#it);
                });
            }
        }

        // load all top-level envs
        let envs = &struct_args.env;
        for Env { name, value } in envs {
            output.extend(quote!(
                cmd.env(#name,#value);
            ));
        }

        // handle struct fields
        for field in data_struct.fields {
            let field_args = match FieldAttributes::from_field(&field) {
                Ok(v) => v,
                Err(e) => {
                    return TokenStream::from(e.write_errors());
                }
            };

            // validate field attributes
            errors = combine_errors(errors, field_args.validate_field(&field));

            // skipped field
            if field_args.skip {
                continue;
            }

            let f_ident = field.ident.unwrap();

            // pass value directly
            if field_args.pass_through {
                output.extend(quote! {
                    cmd.arg(value.#f_ident);
                });
                continue;
            }

            let field_str = f_ident.to_string();

            // get name based on:
            //   1. direct rename
            //   2. field level rename_all
            //   3. struct level rename_all
            //   4. field name
            let name = if let Some(rename) = field_args.rename {
                rename
            } else if let Some(rename_all) = field_args.rename_all {
                rename_all.apply_to_field(&field_str[..])
            } else if let Some(rename_all) = struct_args.rename_all.clone() {
                rename_all.apply_to_field(&field_str[..])
            } else {
                field_str
            };

            // set env based on name and string value
            if field_args.env {
                output.extend(quote! {
                    cmd.env(#name, format!("{}",value.#f_ident));
                });
                continue;
            }

            // get prefix based on:
            //   1. field level prefix
            //   2. struct level prefix
            //   3. default to `--`
            let prefix = if let Some(v) = field_args.prefix {
                v
            } else if let Some(v) = struct_args.prefix.clone() {
                v
            } else {
                "--".to_string()
            };

            // handle bool and Option<bool>, just print name if true
            // todo: do we need to handle more than just bool and Option<bool>?
            // todo: do we need an attribute to pass true/false in or does
            // string suffice?
            if is_bool_type(&field.ty) {
                output.extend(quote! {
                    if value.#f_ident {
                        cmd.arg(format!("{}{}",#prefix,#name));
                    }
                });
                continue;
            } else if is_option_bool_type(&field.ty) {
                output.extend(quote! {
                    if let Some(v) = value.#f_ident && v {
                        cmd.arg(format!("{}{}",#prefix,#name));
                    }
                });
                continue;
            }

            // get delimiter based on:
            //   1. field level delimiter
            //   2. struct level delimiter
            //   3. default to ` `
            let delimiter = if let Some(v) = field_args.delimiter {
                v
            } else if let Some(v) = struct_args.delimiter.clone() {
                v
            } else {
                " ".to_string()
            };

            // if delimiter is a space, it has be added as a separate arg, if it
            // is something like `=` it can be used directly with no spaces
            // todo: is there a better way to handle this?
            if !delimiter.is_empty() && delimiter.trim().is_empty() {
                output.extend(quote! {
                    cmd.arg(format!("{}{}",#prefix,#name));
                    cmd.arg(format!("{}",value.#f_ident));
                });
            } else {
                output.extend(quote! {
                    cmd.arg(format!("{}{}{}{}",#prefix,#name,#delimiter,value.#f_ident));
                });
            }
        }

        // create the implementation of From on Command using the output.
        return quote! {
            #[automatically_derived]
            impl From<#ident> for std::process::Command {
                fn from(value: #ident) -> Self {
                    #output

                    cmd
                }
            }
        }
        .into();
    } else {
        panic!("Commandable can only be used with named structs")
    }
}

/**
 * represents an environmental variable
 */
#[derive(FromMeta, Clone)]
struct Env {
    name: String,
    value: String,
}

/**
 * attributes which can be defined at the struct level
 */
#[derive(FromDeriveInput)]
#[darling(attributes(oocli), supports(struct_named))]
struct StructAttributes {
    // prefix to use for flags, default is `--`
    prefix: Option<String>,
    // delimiter to use for flag name=value, default is a space
    delimiter: Option<String>,
    // the primary command i.e. binary. plus space separated sub commands.
    command: String,
    // optional env binding
    #[darling(default, multiple)]
    env: Vec<Env>,
    // rename all fields based on the same rule. kebab-case is popular for flags
    rename_all: Option<RenameRule>,
}

impl Default for StructAttributes {
    fn default() -> Self {
        Self {
            prefix: None,
            delimiter: None,
            command: "".to_string(),
            env: Vec::new(),
            rename_all: None,
        }
    }
}

/**
 * attributes which can be defined at the field level
 */
#[derive(FromField)]
#[darling(attributes(oocli))]
struct FieldAttributes {
    // prefix to use, default is `--`
    prefix: Option<String>,
    // delimiter to use , default is a space
    delimiter: Option<String>,
    // rename the field.
    #[darling(default)]
    rename: Option<String>,
    // rename the field based on the rule. kebab-case is popular for flags
    #[darling(default)]
    rename_all: Option<RenameRule>,
    // pass the value in as an arg without any name
    #[darling(default)]
    pass_through: bool,
    // use name and value as env
    #[darling(default)]
    env: bool,
    // skip this field
    #[darling(default)]
    skip: bool,
}
impl Default for FieldAttributes {
    fn default() -> Self {
        Self {
            prefix: None,
            delimiter: None,
            rename: None,
            rename_all: None,
            pass_through: false,
            env: false,
            skip: false,
        }
    }
}

impl FieldAttributes {
    fn validate_field(&self, field: &Field) -> Option<Error> {
        let mut errors: Option<Error> = None;

        // skip conflicts
        if self.skip {
            if self.prefix.is_some() {
                let err = Error::new_spanned(&field.ident, "`skip`: `prefix` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.delimiter.is_some() {
                let err =
                    Error::new_spanned(&field.ident, "`skip`: `delimiter` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.rename.is_some() {
                let err = Error::new_spanned(&field.ident, "`skip`: `rename` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.rename_all.is_some() {
                let err =
                    Error::new_spanned(&field.ident, "`skip`: `rename_all` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.pass_through {
                let err =
                    Error::new_spanned(&field.ident, "`skip`: `pass_through` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.env {
                let err = Error::new_spanned(&field.ident, "`skip`: `env` will have no effect");
                errors = combine_errors(errors, Some(err));
            }
        }

        // pass_through conflicts
        if self.pass_through {
            if self.prefix.is_some() {
                let err = Error::new_spanned(
                    &field.ident,
                    "`pass_through`: `prefix` will have no effect",
                );
                errors = combine_errors(errors, Some(err));
            }

            if self.delimiter.is_some() {
                let err = Error::new_spanned(
                    &field.ident,
                    "`pass_through`: `delimiter` will have no effect",
                );
                errors = combine_errors(errors, Some(err));
            }

            if self.rename.is_some() {
                let err = Error::new_spanned(
                    &field.ident,
                    "`pass_through`: `rename` will have no effect",
                );
                errors = combine_errors(errors, Some(err));
            }

            if self.rename_all.is_some() {
                let err = Error::new_spanned(
                    &field.ident,
                    "`pass_through`: `rename_all` will have no effect",
                );
                errors = combine_errors(errors, Some(err));
            }

            if self.env {
                let err =
                    Error::new_spanned(&field.ident, "`pass_through`: `env` will have no effect");
                errors = combine_errors(errors, Some(err));
            }
        }

        // rename conflicts
        if self.rename.is_some() {
            if self.rename_all.is_some() {
                let err =
                    Error::new_spanned(&field.ident, "`rename`: `rename_all` will have no effect");
                errors = combine_errors(errors, Some(err));
            }
        }

        // env conflicts
        if self.env {
            if self.prefix.is_some() {
                let err = Error::new_spanned(&field.ident, "`env`: `prefix` will have no effect");
                errors = combine_errors(errors, Some(err));
            }

            if self.delimiter.is_some() {
                let err =
                    Error::new_spanned(&field.ident, "`env`: `delimiter` will have no effect");
                errors = combine_errors(errors, Some(err));
            }
        }

        return errors;
    }
}

/*
 * this is shamelessly taken from the RenameRule implementation by serde over at
 * https://github.com/serde-rs/serde
 *
 */
#[derive(FromMeta, Debug, Clone)]
enum RenameRule {
    #[darling(rename = "lowercase")]
    Lowercase,
    #[darling(rename = "UPPERCASE")]
    Uppercase,
    #[darling(rename = "PascalCase")]
    PascalCase,
    #[darling(rename = "camelCase")]
    CamelCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    #[darling(rename = "kebab-case")]
    KebabCase,
    #[darling(rename = "SCREAMING-KEBAB-CASE")]
    ScreamingKebabCase,
}

/*
 * this is shamelessly taken from the RenameRule implementation by serde over at
 * https://github.com/serde-rs/serde
 *
 * note: this makes the assumption that the dev named the struct fields following
 * proper snake_case.
 * todo: perhaps there should be some level of validation here?
 */
impl RenameRule {
    pub fn apply_to_field(self, field: &str) -> String {
        match self {
            Lowercase | SnakeCase => field.to_owned(),
            Uppercase => field.to_ascii_uppercase(),
            // assumed to start with snake_case
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            // convert to PascalCase then change the first letter
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }
}

/**
 * checks if a type is `bool`
 */
fn is_bool_type(ty: &Type) -> bool {
    let type_path = match ty {
        Type::Path(path) => &path.path,
        _ => return false,
    };

    // Check for direct `bool` type
    type_path.is_ident("bool")
}

/**
 * checks if a type is `Option<bool>`
 */
fn is_option_bool_type(ty: &Type) -> bool {
    let type_path = match ty {
        Type::Path(path) => &path.path,
        _ => return false,
    };

    // Check for `Option<...>` type
    if type_path.segments.len() == 1 && type_path.segments[0].ident == "Option" {
        if let syn::PathArguments::AngleBracketed(args) = &type_path.segments[0].arguments {
            if args.args.len() == 1 {
                if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                    return is_bool_type(inner_ty);
                }
            }
        }
    }

    false
}

/**
 * combine the errors or init as a new one
 */
fn combine_errors(errors: Option<Error>, err: Option<Error>) -> Option<Error> {
    if let Some(err) = err {
        if let Some(mut errors) = errors {
            errors.combine(err);
            return Some(errors);
        }
        return Some(err);
    }
    return errors;
}
