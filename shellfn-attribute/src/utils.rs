use syn::{Type, TypePath};

pub fn is_result_type(typ: &Type) -> bool {
    if let Type::Path(ref type_path) = *typ {
        is_path_to("Result", type_path)
    } else {
        false
    }
}

pub fn is_unit_type(typ: &Type) -> bool {
    if let Type::Tuple(ref tuple) = typ {
        return tuple.elems.is_empty();
    }

    false
}

pub fn is_vec_type(typ: &Type) -> bool {
    if let Type::Path(ref type_path) = *typ {
        is_vec_type_path(type_path)
    } else {
        false
    }
}

pub fn is_result_type_path(type_path: &TypePath) -> bool {
    is_path_to("Result", type_path)
}

pub fn is_vec_type_path(type_path: &TypePath) -> bool {
    is_path_to("Vec", type_path)
}

fn is_path_to(name: &str, type_path: &TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .is_some_and(|s| s.value().ident == name)
}
