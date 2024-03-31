#[macro_export]
macro_rules! engines {
    ($($engine:ident = $id:expr),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Engine {
            $($engine,)*
        }

        impl Engine {
            #[must_use]
            pub fn all() -> &'static [Engine] {
                &[$(Engine::$engine,)*]
            }

            #[must_use]
            pub fn id(&self) -> &'static str {
                match self {
                    $(Engine::$engine => $id,)*
                }
            }

            #[must_use]
            pub fn id_proper(&self) -> &'static str {
                match self {
                    $(Engine::$engine => stringify!($engine),)*
                }
            }

            #[must_use]
            pub fn from_id(id: &str) -> Option<Engine> {
                match id {
                    $($id => Some(Engine::$engine),)*
                    _ => None
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_weights {
    ($($engine:ident = $weight:expr),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn weight(&self) -> f64 {
                match self {
                    $(Engine::$engine => $weight,)*
                    _ => 1.,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_scholarly {
    ($($engine:ident = $is_scholarly:expr),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn is_scholarly(&self) -> bool {
                match self {
                    $(Engine::$engine => $is_scholarly,)*
                    _ => false,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_enabled {
    ($($engine:ident = $is_enabled_by_default:expr),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn is_enabled(&self, enabled_engines: &std::collections::BTreeMap<String, bool>) -> bool {
                *enabled_engines.get(self.id()).unwrap_or(&self.is_enabled_by_default())
            }

            #[must_use]
            pub fn is_enabled_by_default(&self) -> bool {
                match self {
                    $(Engine::$engine => $is_enabled_by_default,)*
                    _ => true,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_parse_response {
    ($res:ident, $module:ident::$engine_id:ident::None) => {
        None
    };
    ($res:ident, $module:ident::$engine_id:ident::$parse_response:ident) => {
        Some($module::$engine_id::$parse_response($res.into()))
    };
}

#[macro_export]
macro_rules! engine_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn request(&self, query: &SearchQuery) -> RequestResponse {
                #[allow(clippy::useless_conversion)]
                match self {
                    $(
                        Engine::$engine => $module::$engine_id::$request(query).into(),
                    )*
                    _ => RequestResponse::None,
                }
            }

            pub fn parse_response(&self, res: &HttpResponse) -> eyre::Result<EngineResponse> {
                #[allow(clippy::useless_conversion)]
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { res, $module::$engine_id::$parse_response }
                            .ok_or_else(|| eyre::eyre!("engine {self:?} can't parse response"))?,
                    )*
                    _ => eyre::bail!("engine {self:?} can't parse response"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_autocomplete_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn request_autocomplete(&self, query: &str) -> Option<RequestAutocompleteResponse> {
                match self {
                    $(
                        Engine::$engine => Some($module::$engine_id::$request(query).into()),
                    )*
                    _ => None,
                }
            }

            pub fn parse_autocomplete_response(&self, body: &str) -> eyre::Result<Vec<String>> {
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { body, $module::$engine_id::$parse_response }
                            .ok_or_else(|| eyre::eyre!("engine {self:?} can't parse autocomplete response"))?,
                    )*
                    _ => eyre::bail!("engine {self:?} can't parse autocomplete response"),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! engine_postsearch_requests {
    ($($engine:ident => $module:ident::$engine_id:ident::$request:ident, $parse_response:ident),* $(,)?) => {
        impl Engine {
            #[must_use]
            pub fn postsearch_request(&self, response: &Response) -> Option<reqwest::RequestBuilder> {
                match self {
                    $(
                        Engine::$engine => $module::$engine_id::$request(response),
                    )*
                    _ => None,
                }
            }

            #[must_use]
            pub fn postsearch_parse_response(&self, res: &HttpResponse) -> Option<String> {
                match self {
                    $(
                        Engine::$engine => $crate::engine_parse_response! { res, $module::$engine_id::$parse_response }?,
                    )*
                    _ => None,
                }
            }
        }
    };
}
