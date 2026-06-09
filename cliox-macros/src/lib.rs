/**
 * oocli provides the Commandable trait and oocli attribute to define how a struct
 * can be turned into a std::process::Command
 */
extern crate proc_macro2;

mod parser;
use proc_macro::TokenStream;

#[proc_macro_derive(Commandable, attributes(cliox))]
pub fn derive_commandable(item: TokenStream) -> TokenStream {
    parser::derive_cliox_impl(item)
}
