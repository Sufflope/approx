#![no_std]

extern crate alloc;
#[macro_use]
extern crate approx;

#[cfg(feature = "derive")]
mod test_derive {
    use approx::{AbsDiffEq, RelativeEq};

    const BAR: &str = "bar";
    const FOO: &str = "foo";

    #[test]
    fn mixed_enum() {
        #[derive(AbsDiffEq, Clone, Debug, PartialEq, RelativeEq)]
        #[approx(epsilon = f32, absolute = 0.1, relative = 0.01)]
        enum Mixed {
            Unit,
            Unnamed(
                #[approx(approximate)] f32,
                &'static str,
                #[approx(skip)] bool,
            ),
            Named {
                #[approx(approximate)]
                value: f32,
                text: &'static str,
                #[approx(skip)]
                ignored: bool,
            },
        }

        impl Mixed {
            fn value(&self, value: f32) -> Self {
                let result = self.clone();
                match result {
                    Self::Unit => result,
                    Self::Unnamed(_, text, ignored) => Self::Unnamed(value, text, ignored),
                    Self::Named { text, ignored, .. } => Self::Named {
                        value,
                        text,
                        ignored,
                    },
                }
            }

            fn text(&self, text: &'static str) -> Self {
                let result = self.clone();
                match result {
                    Self::Unit => result,
                    Self::Unnamed(value, _, ignored) => Self::Unnamed(value, text, ignored),
                    Self::Named { value, ignored, .. } => Self::Named {
                        value,
                        text,
                        ignored,
                    },
                }
            }

            fn ignored(&self, ignored: bool) -> Self {
                let result = self.clone();
                match result {
                    Self::Unit => result,
                    Self::Unnamed(value, text, _) => Self::Unnamed(value, text, ignored),
                    Self::Named { value, text, .. } => Self::Named {
                        value,
                        text,
                        ignored,
                    },
                }
            }
        }

        // Unit variants are tautologically equal
        assert_abs_diff_eq!(Mixed::Unit, Mixed::Unit);
        assert_relative_eq!(Mixed::Unit, Mixed::Unit);

        // Reference named variant
        let named = Mixed::Named {
            value: 1000.,
            text: FOO,
            ignored: true,
        };

        // A small difference both absolutely and relatively acceptable (and the boolean is ignored)
        let named_both = named.value(1000.01).ignored(false);
        assert_abs_diff_eq!(named, named_both);
        assert_relative_eq!(named, named_both);

        // A larger difference only relatively acceptable
        let named_relative = named.value(1001.).ignored(false);
        assert_abs_diff_ne!(named, named_relative);
        assert_relative_eq!(named, named_relative);

        // Too different
        let named_different = named.value(0.);
        assert_abs_diff_ne!(named, named_different);
        assert_relative_ne!(named, named_different);

        // The string must be exactly equal
        let named_different_string = named.text(BAR);
        assert_abs_diff_ne!(named, named_different_string);
        assert_relative_ne!(named, named_different_string);

        // Reference unnamed variant
        let unnamed = Mixed::Unnamed(1000., FOO, true);

        // A small difference both absolutely and relatively acceptable (and the boolean is ignored)
        let unnamed_both = unnamed.value(1000.01).ignored(false);
        assert_abs_diff_eq!(unnamed, unnamed_both);
        assert_relative_eq!(unnamed, unnamed_both);

        // A larger difference only relatively acceptable
        let unnamed_relative = unnamed.value(1001.).ignored(false);
        assert_abs_diff_ne!(unnamed, unnamed_relative);
        assert_relative_eq!(unnamed, unnamed_relative);

        // Too different
        let unnamed_different = unnamed.value(0.);
        assert_abs_diff_ne!(unnamed, unnamed_different);
        assert_relative_ne!(unnamed, unnamed_different);

        // The string must be exactly equal
        let unnamed_different_string = unnamed.text(BAR);
        assert_abs_diff_ne!(unnamed, unnamed_different_string);
        assert_relative_ne!(unnamed, unnamed_different_string);

        // Different variants are not comparable
        assert_abs_diff_ne!(Mixed::Unit, named);
        assert_relative_ne!(Mixed::Unit, named);
        assert_abs_diff_ne!(Mixed::Unit, unnamed);
        assert_relative_ne!(Mixed::Unit, unnamed);
        assert_abs_diff_ne!(named, unnamed);
        assert_relative_ne!(named, unnamed);
    }

    #[test]
    fn named_struct() {
        #[derive(AbsDiffEq, Clone, Debug, PartialEq, RelativeEq)]
        #[approx(epsilon = f32, absolute = Named::EPSILON, relative = 0.01)]
        struct Named {
            #[approx(approximate)]
            value: f32,
            text: &'static str,
            #[approx(skip)]
            ignored: bool,
        }

        impl Named {
            const EPSILON: f32 = 0.1;

            fn value(&self, value: f32) -> Self {
                Self {
                    value,
                    ..self.clone()
                }
            }

            fn text(&self, text: &'static str) -> Self {
                Self {
                    text,
                    ..self.clone()
                }
            }

            fn ignored(&self, ignored: bool) -> Self {
                Self {
                    ignored,
                    ..self.clone()
                }
            }
        }

        let reference = Named {
            value: 1000.,
            text: FOO,
            ignored: true,
        };

        let both = reference.value(1000.01).ignored(false);
        assert_abs_diff_eq!(reference, both);
        assert_relative_eq!(reference, both);

        let relative = reference.value(1001.).ignored(false);
        assert_abs_diff_ne!(reference, relative);
        assert_relative_eq!(reference, relative);

        let different = reference.value(0.);
        assert_abs_diff_ne!(reference, different);
        assert_relative_ne!(reference, different);

        let exact = reference.text(BAR);
        assert_abs_diff_ne!(reference, exact);
        assert_relative_ne!(reference, exact);
    }

    #[test]
    fn unnamed_struct() {
        const fn epsilon() -> f64 {
            0.1
        }

        #[derive(AbsDiffEq, Debug, PartialEq, RelativeEq)]
        #[approx(epsilon = f64, absolute = epsilon(), relative = 0.01)]
        struct Unnamed(
            #[approx(approximate)] f64,
            &'static str,
            #[approx(skip)] bool,
        );

        impl Unnamed {
            fn value(&self, value: f64) -> Self {
                Self(value, self.1, self.2.clone())
            }

            fn text(&self, text: &'static str) -> Self {
                Self(self.0.clone(), text, self.2.clone())
            }

            fn ignored(&self, ignored: bool) -> Self {
                Self(self.0.clone(), self.1, ignored)
            }
        }

        let reference = Unnamed(1000., FOO, true);

        let both = reference.value(1000.01).ignored(false);
        assert_abs_diff_eq!(reference, both);
        assert_relative_eq!(reference, both);

        let relative = reference.value(1001.).ignored(false);
        assert_abs_diff_ne!(reference, relative);
        assert_relative_eq!(reference, relative);

        let different = reference.value(0.);
        assert_abs_diff_ne!(reference, different);
        assert_relative_ne!(reference, different);

        let exact = reference.text(BAR);
        assert_abs_diff_ne!(reference, exact);
        assert_relative_ne!(reference, exact);
    }

    #[test]
    fn generic_struct() {
        #[derive(AbsDiffEq, Clone, Debug, PartialEq, RelativeEq)]
        #[approx(epsilon = T::Epsilon, absolute = T::default_epsilon(), relative = T::default_max_relative())]
        struct Gen<T>
        where
            T: RelativeEq,
        {
            #[approx(approximate)]
            value: T,
            text: &'static str,
            #[approx(skip)]
            ignored: bool,
        }

        impl<T> Gen<T>
        where
            T: RelativeEq,
        {
            fn value(&self, value: T) -> Self
            where
                T: Clone,
            {
                Self {
                    value,
                    ..self.clone()
                }
            }
        }

        let reference = Gen {
            value: 100000000.0f32,
            text: FOO,
            ignored: true,
        };
        let other = reference.value(reference.value + 1.);
        assert_abs_diff_eq!(reference, other);
        assert_relative_eq!(reference, other);
    }
}
