#[macro_export]
macro_rules! avm2_stub_method {
    ($activation: ident, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: $class,
            method: $method,
            specifics: None,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
    ($activation: ident, $class: literal, $method: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Method {
            class: $class,
            method: $method,
            specifics: Some($specifics),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_constructor {
    ($activation: ident, $class: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Constructor { class: $class };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_getter {
    ($activation: ident, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Getter {
            class: $class,
            property: $property,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}

#[macro_export]
macro_rules! avm2_stub_setter {
    ($activation: ident, $class: literal, $property: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm2Setter {
            class: $class,
            property: $property,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}
