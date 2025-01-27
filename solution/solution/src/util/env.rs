#[macro_export]
macro_rules! environment_variables {
    ( $( $name:ident: $default_value:expr ),* $(,)? ) => {
        $(
            #[allow(non_snake_case)]
            pub fn $name() -> &'static str {
                static $name: OnceLock<Result<String, env::VarError>> = OnceLock::new();

                let name = stringify!($name);

                if let Ok(value) = $name.get_or_init(|| env::var(name)) {
                    &**value
                } else {
                    warn!(
                        "Variable `{}` is missing in env! Using default_value `{}`",
                        name,
                        $default_value
                    );
                    $default_value
                }
            }
        )*
    };
}
