use syn::{
    parse::{Parse, ParseStream, Result},
    Token,
};

pub fn get_attr_value<P: Parse>(stream: ParseStream) -> Result<P> {
    stream.parse::<Token![=]>()?;
    stream.parse::<P>()
}
