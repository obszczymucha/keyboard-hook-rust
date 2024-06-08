#[macro_export]
macro_rules! t {
    ($key:expr) => {
        Mapping::Single($crate::types::Behaviour::Timeout(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
        ))
    };

    ($key:expr, $modifier:expr) => {
        Mapping::Single($crate::types::Behaviour::Timeout(
            $crate::types::KeyPress::Mod($key, $modifier),
        ))
    };
}

#[macro_export]
macro_rules! c {
    ([$($behaviours:expr),* $(,)?], $tag:expr) => {
        $crate::types::Mapping::Choice($crate::types::Behaviours(vec![$($behaviours),*]), $tag)
    };
}

#[macro_export]
macro_rules! shutdown {
    ($key:expr) => {
        Mapping::Single($crate::types::Behaviour::Shutdown(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
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
macro_rules! key_aot {
    ($key:expr, $action:expr) => {
        $crate::types::Behaviour::ActionOnTimeout(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
            $action,
        )
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        $crate::types::Behaviour::ActionOnTimeout(
            $crate::types::KeyPress::Mod($key, $modifier),
            $action,
        )
    };
}

#[macro_export]
macro_rules! key_a {
    ($key:expr, $action:expr) => {
        $crate::types::Behaviour::Action(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
            $action,
        )
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        $crate::types::Behaviour::Action($crate::types::KeyPress::Mod($key, $modifier), $action)
    };
}

#[macro_export]
macro_rules! key_t {
    ($key:expr) => {
        $crate::types::Behaviour::Timeout($crate::types::KeyPress::Mod(
            $key,
            $crate::types::Modifier::NoMod,
        ))
    };

    ($key:expr, $modifier:expr) => {
        $crate::types::Behaviour::Timeout($crate::types::KeyPress::Mod($key, $modifier))
    };
}

#[macro_export]
macro_rules! aot {
    ($key:expr, $action:expr) => {
        Mapping::Single($crate::types::Behaviour::ActionOnTimeout(
            $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod),
            $action,
        ))
    };

    ($key:expr, $modifier:expr, $action:expr) => {
        Mapping::Single($crate::types::Behaviour::ActionOnTimeout(
            $crate::types::KeyPress::Mod($key, $modifier),
            $action,
        ))
    };
}

#[macro_export]
macro_rules! key {
    ($key:expr) => {
        $crate::types::KeyPress::Mod($key, $crate::types::Modifier::NoMod)
    };
}

#[macro_export]
macro_rules! alt {
    ($key:expr) => {
        $crate::types::KeyPress::Mod($key, $crate::types::Modifier::ModAlt)
    };
}
