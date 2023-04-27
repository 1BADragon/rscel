#[macro_export]
macro_rules! enum_str {
    ($vis:vis enum $name:ident {
        $($variant:ident $(= $val:expr)?),*,
    }) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        $vis enum $name {
            $($variant $(= $val)?),*
        }

        impl $name {
            pub fn from_str(s: &str) -> Result<$name, ()> {
                match s {
                    $(stringify!($variant) => Ok($name::$variant)),*,
                    _ => Err(())
                }
            }

            pub fn to_str(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
}
