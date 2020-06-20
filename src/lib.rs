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
    (
        $callback:ident;

        $( #[$attr:meta] )*
        fn $name:ident(&mut self $(, $arg_name:ident : $arg_ty:ty )*) $(-> $ret:ty)?;

        $( $rest:tt )*
    ) => {
        $callback!(
            $( #[$attr] )*
            fn $name(&mut self $(, $arg_name : $arg_ty )*) $(-> $ret)?
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
    (
        $( #[$attr:meta] )*
        fn $name:ident(&mut self $(, $arg_name:ident : $arg_ty:ty )*) $(-> $ret:ty)?
    ) => {
        fn $name(&mut self $(, $arg_name : $arg_ty )*) $(-> $ret)? {
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
        impl<'f, F: $name + ?Sized> $name for &'f F {
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
        impl<'f, F: $name + ?Sized> $name for &'f mut F {
            visit_members!( call_via_deref; $($body)* );
        }
    };
}

/// Scans through a stream of tokens looking for `&mut self`. If nothing is
/// found a callback is invoked.
#[macro_export]
macro_rules! search_for_mut_self {
    // if we see `&mut self`, stop and don't invoke the callback
    ($callback:ident!($($callback_args:tt)*); &mut self $($rest:tt)*) => { };
    ($callback:ident!($($callback_args:tt)*); (&mut self $($other_args:tt)*) $($rest:tt)*) => { };

    // haven't found it yet, drop the first item and keep searching
    ($callback:ident!($($callback_args:tt)*); $_head:tt $($tokens:tt)*) => {
        search_for_mut_self!($callback!( $($callback_args)* ); $($tokens)*);

    };
    // we completed without hitting `&mut self`, invoke the callback and exit
    ($callback:ident!($($callback_args:tt)*);) => {
        $callback!( $($callback_args)* )
    }
}

#[macro_export]
macro_rules! trait_with_dyn_impls {
    (
        $( #[$attr:meta] )*
        $vis:vis trait $name:ident { $( $body:tt )* }
    ) => {
        // emit the trait declaration
        $( #[$attr] )*
        $vis trait $name { $( $body )* }

        impl_trait_for_mut_ref! {
            $( #[$attr] )*
            $vis trait $name { $( $body )* }
        }
        impl_trait_for_boxed! {
            $( #[$attr] )*
            $vis trait $name { $( $body )* }
        }

        // we can only implement the trait for `&T` if there are NO `&mut self`
        // methods
        search_for_mut_self! {
            impl_trait_for_ref!( $( #[$attr] )* $vis trait $name { $( $body )* } );

            $( $body )*
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

    #[test]
    fn full_implementation() {
        trait_with_dyn_impls! {
            trait Foo {
                fn get_x(&self) -> u32;
                fn execute(&self, expression: &str);
            }
        }

        fn assert_is_foo<F: Foo>() {}

        assert_is_foo::<&dyn Foo>();
        assert_is_foo::<Box<dyn Foo>>();
    }

    #[test]
    fn handle_mutable_and_immutable_self() {
        trait Foo {
            fn get_x(&self) -> u32;
            fn execute(&mut self, expression: &str);
        }

        impl_trait_for_boxed! {
            trait Foo {
                fn get_x(&self) -> u32;
                fn execute(&mut self, expression: &str);
            }
        }
    }

    #[test]
    fn full_implementation_with_mut_methods() {
        trait_with_dyn_impls! {
            trait Foo {
                fn execute(&mut self, expression: &str);
            }
        }

        fn assert_is_foo<F: Foo>() {}

        assert_is_foo::<&mut dyn Foo>();
        assert_is_foo::<Box<dyn Foo>>();
    }

    #[test]
    fn dont_invoke_the_callback_when_mut_self_found() {
        search_for_mut_self! {
            compile_error!("This callback shouldn't have been invoked");

            &mut self asdf
        }
    }

    #[test]
    fn handle_mut_self_inside_parens() {
        search_for_mut_self! {
            compile_error!("This callback shouldn't have been invoked");

            fn foo(&mut self);
        }
    }

    #[test]
    fn invoke_the_callback_if_search_for_mut_self_found() {
        macro_rules! declare_struct {
            ($name:ident) => {
                struct $name;
            };
        }

        search_for_mut_self! {
            declare_struct!(Foo);

            blah blah ... blah
        }

        // we should have declared Foo as a unit struct
        let _: Foo;
    }
}
