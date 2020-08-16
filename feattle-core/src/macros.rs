#[macro_export]
macro_rules! feattle_enum {
    ($key:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
        #[derive($crate::__internal::EnumString)]
        #[derive($crate::__internal::EnumVariantNames)]
        #[derive($crate::__internal::Display)]
        pub enum $key { $($variant),* }

        impl $crate::FeattleStringValue for $key {
            fn serialized_string_format() -> StringFormat {
                StringFormat::Choices(&$key::VARIANTS)
            }
        }
    }
}

#[macro_export]
macro_rules! __init_field {
    ($default:expr) => {
        $default
    };
    () => {
        Default::default()
    };
}

#[macro_export]
macro_rules! feattles {
    (
    $name:ident {
        $(
            $(#[doc=$description:tt])*
            $key:ident: $type:ty $(= $default:expr)?
        ),*
        $(,)?
    }
    ) => {
        pub mod __feattles {
            use feattle_core::__internal;
            use super::*;

            pub struct $name<P>(__internal::FeattlesImpl<P, __Features>);
            pub struct __Features {
                $($key: __internal::Feature<$type>),*
            }

            impl __internal::FeaturesStruct for __Features {
                fn update(
                    &mut self,
                    key: &str,
                    value: &__internal::CurrentValue,
                ) -> Result<(), __internal::FromJsonError> {
                    match key {
                        $(stringify!($key) => self.$key.update(value)),*,
                        _ => unreachable!(),
                    }
                }
            }

            impl<P: __internal::Persist> __internal::Feattles<P> for $name<P> {
                type FeatureStruct = __Features;

                fn _read(
                    &self,
                ) -> __internal::RwLockReadGuard<'_, __internal::InnerFeattles<Self::FeatureStruct>>
                {
                    self.0.inner_feattles.read()
                }

                fn _write(
                    &self,
                ) -> __internal::RwLockWriteGuard<'_, __internal::InnerFeattles<Self::FeatureStruct>>
                {
                    self.0.inner_feattles.write()
                }

                fn new(persistence: P) -> Self {
                    $name(__internal::FeattlesImpl::new(
                        persistence,
                        __Features {
                            $(
                                $key: __internal::Feature::new(
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

                fn definition(&self, key: &str) -> Option<__internal::FeatureDefinition> {
                    let inner = self._read();
                    match key {
                        $(stringify!($key) => Some(inner.feattles_struct.$key.definition())),*,
                        _ => None,
                    }
                }
            }

            impl<P: __internal::Persist> $name<P> {
                $(
                    pub fn $key(&self) -> __internal::MappedRwLockReadGuard<$type> {
                        __internal::RwLockReadGuard::map(self.0.inner_feattles.read(), |inner| {
                            &inner.feattles_struct.$key.value
                        })
                    }
                )*
            }
        }

        use __feattles::$name;
    }
}
