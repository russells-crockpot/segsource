#[macro_export]
macro_rules! make_add_macro {
    (
        ($d:tt)
        name: $name_:ident;
        type: $type_:path;
        body: {$($body:item)+}
    ) => {
        #[macro_export]
        macro_rules! $name_ {
            ($type:path) => {
                impl $type_ for $type { $($body)+ }
            };

            ($type:path, $d($lifetimes:lifetime),+ $d($generics:ident),*) => {
                    impl<$d($lifetimes,)+ $d($generics,)*> $type_ for $type { $($body)+ }
            }
        }
    };
    (
        name: $name:ident;
        type: $type:path;
        body: {$($body:item)+}
    ) => {
        make_add_macro! {
            ($)
            name: $name;
            type: $type;
            body: { $($body)+ }
        }
    }
}
