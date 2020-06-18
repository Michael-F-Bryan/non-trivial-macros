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

#[macro_export]
macro_rules! call_via_deref {
    (
        $( #[$attr:meta] )*
        fn $name:ident(&self $(, $arg_name:ident : $arg_ty:ty )*) $(-> $ret:ty)?
    ) => {
        fn $name(&self $(, $arg_name : $arg_ty )*) $(-> $ret)? {
            (**self).$name( $($arg_name),* )
        }
    };
}

#[macro_export]
macro_rules! impl_trait_for_boxed {
    (
        $( #[$attr:meta] )*
        $vis:vis trait $name:ident {
            $( $body:tt )*
        }
    ) => {
        impl<F: $name + ?Sized> $name for Box<F> {
            visit_members!( call_via_deref; $($body)* );
        }
    };
}

#[macro_export]
macro_rules! impl_trait_for_ref {
    (
        $( #[$attr:meta] )*
        $vis:vis trait $name:ident {
            $( $body:tt )*
        }
    ) => {
        impl<'f, F: $name + ?Sized> $name for &'f dyn F {
            visit_members!( call_via_deref; $($body)* );
        }
    };
}

#[macro_export]
macro_rules! impl_trait_for_mut_ref {
    (
        $( #[$attr:meta] )*
        $vis:vis trait $name:ident {
            $( $body:tt )*
        }
    ) => {
        impl<'f, F: $name + ?Sized> $name for &'f mut dyn F {
            visit_members!( call_via_deref; $($body)* );
        }
    };
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

    #[test]
    fn defer_to_item_behind_pointer() {
        trait GetX {
            fn get_x(&self) -> u32;
        }

        impl GetX for u32 {
            fn get_x(&self) -> u32 { *self }
        }

        impl GetX for Box<u32> {
            call_via_deref!( fn get_x(&self) -> u32 );
        }

        fn assert_is_get_x<G: GetX>() {}

        assert_is_get_x::<u32>();
        assert_is_get_x::<Box<u32>>();
    }

    #[test]
    fn impl_trait_for_boxed() {
        trait Foo {
            fn get_x(&self) -> u32;
            fn execute(&self, expression: &str);
        }

        impl Foo for u32 {
            fn get_x(&self) -> u32 { unimplemented!() }

            fn execute(&self, _expression: &str) { unimplemented!() }
        }

        impl_trait_for_boxed! {
            trait Foo {
                fn get_x(&self) -> u32;
                fn execute(&self, expression: &str);
            }
        }

        fn assert_is_foo<F: Foo>() {}

        assert_is_foo::<u32>();
        assert_is_foo::<Box<u32>>();
        assert_is_foo::<Box<dyn Foo>>();
    }
}
