#[macro_export]
macro_rules! t {
    ($key:expr) => {
        Mapping::Timeout($crate::types::KeyPressType::Single(
            $crate::types::KeyPress::nomod($key),
        ))
    };

    ($key:expr, $modifier:expr) => {
        Mapping::Timeout($crate::types::KeyPressType::Single(
            $crate::types::KeyPress::new($key, $modifier),
        ))
    };
}

#[allow(dead_code)]
#[macro_export]
macro_rules! tm {
    ([$($keypresses:expr),* $(,)?]) => {
        Mapping::Timeout(KeyPresses(vec![$($keypresses),*]).choice())
    };
}

#[macro_export]
macro_rules! a {
    ($key:expr, $action_type:expr) => {
        Mapping::Action(
            $crate::types::KeyPressType::Single($crate::types::KeyPress::nomod($key)),
            $crate::types::ActionMapping::NoTimeout($action_type),
        )
    };

    ($key:expr, $modifier:expr, $action_type:expr) => {
        Mapping::Action(
            $crate::types::KeyPressType::Single($crate::types::KeyPress::new($key, $modifier)),
            $crate::types::ActionMapping::NoTimeout($action_type),
        )
    };
}

#[macro_export]
macro_rules! aat {
    ([$($keypresses:expr),* $(,)?], $action_type:expr) => {
        Mapping::Action($crate::types::KeyPresses(vec![$($keypresses),*]).choice(), $crate::types::ActionMapping::TimeoutBeforeAction($action_type))
    }
}

#[macro_export]
macro_rules! abt {
    ([$($keypresses:expr),* $(,)?], $action_type:expr) => {
        Mapping::Action($crate::types::KeyPresses(vec![$($keypresses),*]).choice(), $crate::types::ActionMapping::TimeoutAfterAction($action_type))
    };

    ($key:expr, $action:expr) => {
        Mapping::Action(Single(KeyPress::nomod($key)), $crate::types::ActionMapping::TimeoutAfterAction($action))
    };
}

#[macro_export]
macro_rules! key {
    ($key:expr) => {
        $crate::types::KeyPress::nomod($key)
    };
}

#[macro_export]
macro_rules! alt {
    ($key:expr) => {
        $crate::types::KeyPress::alt($key)
    };
}
