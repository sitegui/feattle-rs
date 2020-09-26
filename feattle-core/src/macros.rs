/// Define an `enum` that can be used as a type for a feattle
///
/// The generated `enum` will have these standard traits: `Debug`, `Clone`, `Copy`, `Eq`,
/// `PartialEq`, `PartialOrd`, `Ord`, `FromStr`, `Display`. And mainly, it will implement
/// [`crate::FeattleStringValue`] so that it can be used a feattle type.
///
/// Only `enum`s whose variants do not carry any extra information are supported.
///
/// # Examples
/// In the simplest form:
/// ```
/// use feattle_core::feattle_enum;
///
/// feattle_enum! {
///     enum Colors { Red, Green, Blue }
/// }
/// ```
///
/// However, it also works with other visibility keywords and additional attributes on the enum
/// itself or its variants. Those attributes will not be modified by this lib, allowing composition
/// with other libs. For example, you can also make the enum `Serialize`:
/// ```
/// use feattle_core::feattle_enum;
/// use serde::Serialize;
///
/// feattle_enum! {
///     #[derive(Serialize)]
///     pub(crate) enum Colors {
///         #[serde(rename = "R")]
///         Red,
///         #[serde(rename = "G")]
///         Green,
///         #[serde(rename = "B")]
///         Blue,
///     }
/// }
/// ```
#[macro_export]
macro_rules! feattle_enum {
    (
        $(#[$enum_meta:meta])*
        $visibility:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
        $(#[$enum_meta])*
        $visibility enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),+
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::__internal::ParseError;
            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => ::std::result::Result::Ok(Self::$variant)
                    ),+,
                    _ => ::std::result::Result::Err($crate::__internal::ParseError)
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let as_str = match self {
                    $(
                        Self::$variant => stringify!($variant)
                    ),+
                };
                ::std::write!(f, "{}", as_str)
            }
        }

        impl $name {
            const VARIANTS: &'static [&'static str] = &[
                $(
                    stringify!($variant)
                ),+
            ];
        }

        impl $crate::FeattleStringValue for $name {
            fn serialized_string_format() -> $crate::StringFormat {
                let variants = Self::VARIANTS.join(", ");
                $crate::StringFormat {
                    kind: $crate::StringFormatKind::Choices(&Self::VARIANTS),
                    tag: format!("enum {{{}}}", variants),
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __init_field {
    ($default:expr) => {
        $default
    };
    () => {
        Default::default()
    };
}

/// The main macro of this crate, used to generate a struct that will provide the Feattles
/// functionalities.
///
/// See more at the [crate level](crate).
#[macro_export]
macro_rules! feattles {
    (
    $(#[$meta:meta])*
    $visibility:vis struct $name:ident {
        $(
            $(#[doc=$description:tt])*
            $key:ident: $type:ty $(= $default:expr)?
        ),*
        $(,)?
    }
) => {
        mod __feattles {
            use ::feattle_core::__internal;
            use super::*;

            $(#[$meta])*
            #[derive(Debug)]
            pub struct $name<P>(__internal::FeattlesImpl<P, __Feattles>);

            impl<P: __internal::Persist> __internal::FeattlesPrivate<P> for $name<P> {
                type FeattleStruct = __Feattles;

                fn _read(
                    &self,
                ) -> __internal::RwLockReadGuard<'_, __internal::InnerFeattles<Self::FeattleStruct>>
                {
                    self.0.inner_feattles.read()
                }

                fn _write(
                    &self,
                ) -> __internal::RwLockWriteGuard<'_, __internal::InnerFeattles<Self::FeattleStruct>>
                {
                    self.0.inner_feattles.write()
                }
            }

            impl<P: __internal::Persist> __internal::Feattles<P> for $name<P> {
                fn new(persistence: P) -> Self {
                    $name(__internal::FeattlesImpl::new(
                        persistence,
                        __Feattles {
                            $(
                                $key: __internal::Feattle::new(
                                    stringify!($key),
                                    concat!($($description),*).trim(),
                                    $crate::__init_field!($($default)?),
                                )
                            ),*
                        },
                    ))
                }

                fn persistence(&self) -> &P {
                    &self.0.persistence
                }

                fn keys(&self) -> &'static [&'static str] {
                    &[$(stringify!($key)),*]
                }

                fn definition(&self, key: &str) -> Option<__internal::FeattleDefinition> {
                    use __internal::FeattlesPrivate;
                    let inner = self._read();
                    match key {
                        $(stringify!($key) => Some(inner.feattles_struct.$key.definition()),)*
                        _ => None,
                    }
                }
            }

            impl<P: __internal::Persist> $name<P> {
                $(
                    pub fn $key(&self) -> __internal::MappedRwLockReadGuard<$type> {
                        __internal::RwLockReadGuard::map(self.0.inner_feattles.read(), |inner| {
                            inner.feattles_struct.$key.value()
                        })
                    }
                )*
            }

            pub struct __Feattles {
                $($key: __internal::Feattle<$type>),*
            }

            impl __internal::FeattlesStruct for __Feattles {
                fn try_update(
                    &mut self,
                    key: &str,
                    value: Option<__internal::CurrentValue>,
                ) -> Result<Option<__internal::CurrentValue>, __internal::FromJsonError> {
                    match key {
                        $(stringify!($key) => self.$key.try_update(value),)*
                        _ => unreachable!(),
                    }
                }
            }
        }

        $visibility use __feattles::$name;
    }
}
