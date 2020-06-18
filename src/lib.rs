#[macro_export]
macro_rules! visit_members {
    (
        $callback:ident;

        $( #[$attr:meta] )*
        fn $name:ident(&self $(, $arg_name:ident : $arg_ty:ty )*) $(-> $ret:ty)?;

        $( $rest:tt )*
    ) => {
        $callback!(
            $( #[$attr] )*
            fn $name(&self $(, $arg_name : $arg_ty )*) $(-> $ret)?
        );

        visit_members! { $callback; $($rest)* }
    };
    ($callback:ident;) => {};
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! echo {
        ( $($tokens:tt)* ) => {};
    }

    #[test]
    fn visit_simple_getter_method() {
        visit_members! { echo; fn get_x(&self) -> u32; }
    }

    #[test]
    fn visit_method_with_multiple_parameters() {
        visit_members! { echo; fn get_x(&self, foo: usize) -> u32; }
        visit_members! { echo; fn get_x(&self, bar: &str, baz: impl FnOnce()) -> u32; }
    }

    #[test]
    fn visit_method_without_return_type() {
        visit_members! { echo; fn get_x(&self); }
    }

    #[test]
    fn visit_method_with_attributes() {
        visit_members! {
            echo;

            // Get `x`.
            #[allow(bad_style)]
            fn get_x(&self) -> u32;
        }
    }

    #[test]
    fn match_two_getters() {
        visit_members! {
            echo;

            fn get_x(&self) -> u32;
            fn get_y(&self) -> u32;
        }
    }
}
