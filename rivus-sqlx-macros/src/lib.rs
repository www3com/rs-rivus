use proc_macro::TokenStream;

mod sql_macro;

#[proc_macro_attribute]
pub fn sql(args: TokenStream, input: TokenStream) -> TokenStream {
    sql_macro::sql_impl(args, input)
}
