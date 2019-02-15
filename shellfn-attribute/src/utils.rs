use syn::{Type, TypePath};

pub fn is_result_type(typ: &Type) -> bool {
    if let Type::Path(ref type_path) = *typ {
        is_result_type_path(type_path)
    } else {
        false
    }
}

pub fn is_result_type_path(type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map_or(false, |s| s.value().ident.to_string() == "Result")
}
