use proc_macro2::TokenStream;

use crate::method_info::MethodInfo;

pub struct ImplInfo {
    pub source: TokenStream,
    pub name: String,
    pub methods: Vec<MethodInfo>,
}
