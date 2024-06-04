#[macro_export]
macro_rules! t {
    ($key:expr) => {
        Mapping::Single($crate::types::Behaviour::Timeout(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
        ))
    };

    ([$($behaviours:expr),* $(,)?], $tag:expr) => {
        $crate::types::Mapping::Choice($crate::types::Behaviours(vec![$($behaviours),*], $tag))
    };

    ($key:expr, $modifier:expr) => {
        Mapping::Single($crate::types::Behaviour::Timeout(
            $crate::types::KeyPress::Mod($key, $modifier),
        ))
    };
}

#[macro_export]
macro_rules! a {
    ($key:expr, $action:expr) => {
        Mapping::Single($crate::types::Behaviour::Action(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
            $action,
        ))
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        Mapping::Single($crate::types::Behaviour::Action(
            $crate::types::KeyPress::Mod($key, $modifier),
            $action,
        ))
    };
}

#[macro_export]
macro_rules! aot {
    ($key:expr, $action:expr) => {
        Mapping::Single($crate::types::Behaviour::ActionOnTimeout(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
            $action,
        ))
    }; // ([$($behaviours:expr),* $(,)?], $action:expr) => {
       //     Mapping::Single($crate::types::Behaviour::ActionOnTimeout(
       //         $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
       //         $action,
       //     ))
       // };
}

// #[macro_export]
// macro_rules! abt {
//     ([$($keypresses:expr),* $(,)?], $action_type:expr) => {
//         Mapping::Action($crate::types::KeyPresses(vec![$($keypresses),*]).choice(), $crate::types::ActionMapping::TimeoutAfterAction($action_type))
//     };
//
//     ($key:expr, $action:expr) => {
//         Mapping::Action(Single(KeyPress::nomod($key)), $crate::types::ActionMapping::TimeoutAfterAction($action))
//     };
// }

#[macro_export]
macro_rules! key {
    ($key:expr) => {
        $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod)
    };
}

#[macro_export]
macro_rules! a_key {
    ($key:expr, $action:expr) => {
        $crate::types::KeyPressMapping::a($key, $action)
    };
}

#[macro_export]
macro_rules! alt {
    ($key:expr) => {
        $crate::types::KeyPress::Mod($key, $crate::types::Modifier::ModAlt)
    };
}
