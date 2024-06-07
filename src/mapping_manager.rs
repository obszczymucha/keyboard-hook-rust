use crate::key_handler::Buffers;
use crate::key_handler::KeyHandlerAction;
use crate::mapping_trie::MappingTrie;
use crate::Behaviour;
use crate::KeyPress;
use crate::Mapping;
use core::hash::Hash;
use std::fmt::Debug;
use std::fmt::Display;
use KeyHandlerAction::*;
use Mapping::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Actions<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    pub actions: Vec<A>,
    pub tag: Option<T>,
}

impl<A, T> Actions<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    pub fn empty() -> Self {
        Self {
            actions: vec![],
            tag: None,
        }
    }

    #[allow(dead_code)]
    pub fn from(actions: Vec<A>, tag: T) -> Self {
        Self {
            actions,
            tag: Some(tag),
        }
    }

    pub fn get_actions_on_timeout(&self) -> &Vec<A> {
        &self.actions
    }

    pub fn push(&mut self, action: A, tag: T) {
        self.actions.push(action);
        self.tag = Some(tag);
    }

    pub fn push_action(&mut self, action: A) {
        self.actions.push(action);
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.tag = None;
    }

    pub fn get_tag(&self) -> &Option<T> {
        &self.tag
    }
}

impl<A, T> Display for Actions<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Actions(actions: {:?}, tag: {:?})",
            self.actions, self.tag
        )
    }
}

pub fn find_mapping<A, T>(
    key_press: &KeyPress,
    trie: &MappingTrie<A, T>,
    buffers: &mut Buffers<A, T>,
) -> KeyHandlerAction<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
{
    let mapping = trie.find_mapping(key_press, buffers.key_buffer.get_keypresses());

    if let Some(m) = mapping {
        buffers.key_buffer.push(key_press.clone());
        let action = to_handler_action(m, key_press, &mut buffers.actions_on_timeout);

        if let Action(_) = action {
            buffers.key_buffer.clear();
            buffers.actions_on_timeout.clear();
        }

        action
    } else {
        KeyHandlerAction::Nothing
    }
}

pub fn to_handler_action<A, T>(
    mapping: &Mapping<A, T>,
    key: &KeyPress,
    actions: &mut Actions<A, T>,
) -> KeyHandlerAction<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Sync + Send + Hash,
{
    match mapping {
        Single(behaviour) => match behaviour {
            Behaviour::Timeout(_) => Timeout,
            Behaviour::Action(_, action_type) => Action(action_type.clone()),
            Behaviour::ActionOnTimeout(_, action_type) => ActionOnTimeout(action_type.clone()),
            Behaviour::Shutdown(_) => StopTheHook,
        },
        Choice(behaviours, tag) => {
            match behaviours.get_mapping(key) {
                Some(mapping) => match (mapping, actions.get_tag()) {
                    (Behaviour::Timeout(_), None) => Timeout,
                    (Behaviour::Timeout(_), Some(_)) => {
                        let timeout_actions = actions.get_actions_on_timeout();

                        if timeout_actions.len() == 1 {
                            ActionOnTimeout(timeout_actions.first().unwrap().clone())
                        } else {
                            ActionsOnTimeout(actions.clone())
                        }
                    }
                    (Behaviour::Action(_, action), None) => ActionBeforeTimeout(action.clone()),
                    (Behaviour::Action(_, action), Some(_)) => ActionsBeforeAndOnTimeout {
                        before: action.clone(),
                        on: actions.clone(),
                    },
                    (Behaviour::ActionOnTimeout(_, action), None) => {
                        actions.push(action.clone(), tag.clone());
                        ActionOnTimeout(action.clone())
                    }
                    (Behaviour::ActionOnTimeout(_, action), Some(_)) => {
                        actions.push_action(action.clone());
                        ActionsOnTimeout(actions.clone())
                    }
                    (Behaviour::Shutdown(_), _) => StopTheHook,
                },
                None => Nothing, // Should never happen.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Key::*;
    use crate::types::Modifier::ModAlt;
    use crate::*;
    use key_handler::Buffers;
    use key_handler::KeyHandlerAction;
    use mapping_manager::find_mapping;
    use rstest::rstest;

    macro_rules! m {
        ( [ $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ] ) => {
            vec![
                $(
                    vec![$($x),*]
                ),*
            ]
        }
    }

    macro_rules! t_actions {
        ([$($actions:expr),* $(,)?], $tag:expr) => {
            $crate::key_handler::KeyHandlerAction::ActionsOnTimeout(
                $crate::mapping_manager::Actions::from(vec![$($actions),*], $tag),
            )
        };
    }

    macro_rules! actions {
        ($action_before:expr, [$($actions:expr),* $(,)?], $tag:expr) => {
            $crate::key_handler::KeyHandlerAction::ActionsBeforeAndOnTimeout{
                before: $action_before,
                on: $crate::mapping_manager::Actions::from(vec![$($actions),*], $tag),
            }
        };
    }

    #[derive(Eq, Debug, Clone, PartialEq, Hash)]
    #[allow(dead_code)]
    enum TestAction {
        Princess,
        Kenny,
        VolUp,
        VolDown,
        Chan1,
        Chan2,
        Chan3,
        Chan4,
        Chan5,
    }

    use TestAction::*;

    impl Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::Princess => write!(f, "Princess"),
                TestAction::Kenny => write!(f, "Kenny"),
                TestAction::VolUp => write!(f, "VolumeUp"),
                TestAction::VolDown => write!(f, "VolumeDown"),
                TestAction::Chan1 => write!(f, "ToggleChannel1"),
                TestAction::Chan2 => write!(f, "ToggleChannel2"),
                TestAction::Chan3 => write!(f, "ToggleChannel3"),
                TestAction::Chan4 => write!(f, "ToggleChannel4"),
                TestAction::Chan5 => write!(f, "ToggleChannel5"),
            }
        }
    }

    #[derive(Eq, Debug, Clone, PartialEq, Hash)]
    #[allow(dead_code)]
    enum TestTag {
        Volume,
        TogChans,
    }

    use TestTag::*;

    impl Display for TestTag {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestTag::Volume => write!(f, "Volume"),
                TestTag::TogChans => write!(f, "ToggleChannels"),
            }
        }
    }

    #[rstest]
    // Should invoke a timeout for a key without a modifier.
    #[case(m!([[t!(KeyA)]]), &[key!(KeyA)], &[Timeout])]
    // Should do nothing if the key doesn't match.
    #[case(m!([[t!(KeyA)]]), &[key!(KeyX)], &[Nothing])]
    // Should invoke a timeout for a key with a modifier.
    #[case(m!([[t!(KeyA, ModAlt)]]), &[alt!(KeyA)], &[Timeout])]
    // Should do nothing if the modifier doesn't match.
    #[case(m!([[t!(KeyA, ModAlt)]]), &[key!(KeyA)], &[Nothing])]
    // Should invoke an immediate action for a key without a modifier.
    #[case(m!([[a!(KeyA, VolUp)]]), &[key!(KeyA)], &[Action(VolUp)])]
    // Should invoke an immediate action for a key with a modifier.
    #[case(m!([[a!(KeyA, ModAlt, VolUp)]]), &[alt!(KeyA)], &[Action(VolUp)])]
    // Should invoke an action on the timeout for a key without a modifier.
    #[case(m!([[aot!(KeyA, VolUp)]]), &[key!(KeyA)], &[ActionOnTimeout(VolUp)])]
    // Should invoke an action on the timeout for a key with a modifier.
    #[case(m!([[aot!(KeyA, ModAlt, VolUp)]]), &[alt!(KeyA)], &[ActionOnTimeout(VolUp)])]
    // Should do nothing if the first key doesn't match the first one in the mapping sequence.
    #[case(m!([[t!(KeyA), a!(KeyB, VolUp)]]), &[key!(KeyB)], &[Nothing])]
    // Should invoke an immediate action if the sequence of keys match and the last key is mapped
    // to an action.
    #[case(m!([[t!(KeyA), a!(KeyB, VolUp)]]), &[key!(KeyA), key!(KeyB)], &[Timeout, Action(VolUp)])]
    // Should invoke the first action and ignore the second, because action resets the sequence.
    #[case(m!([[a!(KeyA, VolUp), a!(KeyB, VolDown)]]), &[key!(KeyA), key!(KeyB)], &[Action(VolUp), Nothing])]
    // Should aggregate actions on timeout.
    #[case(m!([[t!(KeyA), c!([key_aot!(Key1, Chan1), key_aot!(Key2, Chan2)], TogChans)]]), &[key!(KeyA), key!(Key1), key!(Key2)], &[Timeout, ActionOnTimeout(Chan1), t_actions!([Chan1, Chan2], TogChans)])]
    // Should aggregate actions on timeout (different keypress order).
    #[case(m!([[t!(KeyA), c!([key_aot!(Key1, Chan1), key_aot!(Key2, Chan2)], TogChans)]]), &[key!(KeyA), key!(Key2), key!(Key1)], &[Timeout, ActionOnTimeout(Chan2), t_actions!([Chan2, Chan1], TogChans)])]
    // Should include repeated actions in aggregate actions on timeout.
    #[case(m!([[t!(KeyA), c!([key_aot!(Key1, Chan1), key_aot!(Key2, Chan2)], TogChans)]]), &[key!(KeyA), key!(Key1), key!(Key2), key!(Key1)], &[Timeout, ActionOnTimeout(Chan1), t_actions!([Chan1, Chan2], TogChans), t_actions!([Chan1, Chan2, Chan1], TogChans)])]
    // Should include timeout keys in aggregates.
    #[case(m!([[t!(KeyA), c!([key_t!(Key1), key_aot!(Key2, Chan2)], TogChans)]]), &[key!(KeyA), key!(Key1), key!(Key2), key!(Key1)], &[Timeout, Timeout, ActionOnTimeout(Chan2), ActionOnTimeout(Chan2)])]
    // Should invoke an immediate action and then aggregated actions on timeout.
    #[case(m!([[t!(KeyA), c!([key_aot!(Key1, Chan1), key_aot!(Key2, Chan2), key_a!(Key3, Chan3)], TogChans)]]), &[key!(KeyA), key!(Key1), key!(Key2), key!(Key3)], &[Timeout, ActionOnTimeout(Chan1), t_actions!([Chan1, Chan2], TogChans), actions!(Chan3, [Chan1, Chan2], TogChans)])]
    fn should_match_keys_to_mappings(
        #[case] mappings: Vec<Vec<Mapping<TestAction, TestTag>>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: &[KeyHandlerAction<TestAction, TestTag>],
    ) {
        // Given
        let trie = MappingTrie::from_mappings(&mappings);
        let mut result = vec![];
        let mut buffers = Buffers::new();

        // When
        for key in keypresses {
            result.push(find_mapping(key, &trie, &mut buffers));
        }

        // Thre asd
        assert_eq!(result, expected)
    }

    fn demo_mappings() -> Vec<Vec<Mapping<TestAction, TestTag>>> {
        vec![
            // Alt+A -> E -> X -> I -> T
            vec![
                t!(KeyA, ModAlt),
                t!(KeyE),
                t!(KeyX),
                t!(KeyI),
                shutdown!(KeyT), // Unhook the keyboard and exit.
            ],
            // Alt+A -> Q
            vec![
                t!(KeyA, ModAlt),
                aot!(KeyQ, Princess), // Action on timeout.
            ],
            // Alt+A -> W
            vec![
                t!(KeyA, ModAlt),
                a!(KeyW, Kenny), // Immediate action.
            ],
            // Alt+A -> [12345]*
            vec![
                t!(KeyA, ModAlt),
                c!(
                    [
                        key_aot!(Key1, Chan1),
                        key_aot!(Key2, Chan2),
                        key_aot!(Key3, Chan3),
                        key_aot!(Key4, Chan4),
                        key_aot!(Key5, Chan5)
                    ],
                    TogChans
                ),
            ],
            // Alt+A -> [JK]*
            vec![
                t!(KeyA, ModAlt),
                c!([key_a!(KeyJ, VolDown), key_a!(KeyK, VolUp)], Volume),
            ],
        ]
    }

    #[rstest]
    #[case(demo_mappings(), &[alt!(KeyA)], &[Timeout])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyE)], &[Timeout, Timeout])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyE), key!(KeyX)], &[Timeout, Timeout, Timeout])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyE), key!(KeyX), key!(KeyI)], &[Timeout, Timeout, Timeout, Timeout])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyE), key!(KeyX), key!(KeyI), key!(KeyT)], &[Timeout, Timeout, Timeout, Timeout, StopTheHook])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyQ)], &[Timeout, ActionOnTimeout(Princess)])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyW)], &[Timeout, Action(Kenny)])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(Key1), key!(Key2), key!(Key3), key!(Key4), key!(Key5), key!(Key3)], &[Timeout, ActionOnTimeout(Chan1), t_actions!([Chan1, Chan2], TogChans), t_actions!([Chan1, Chan2, Chan3], TogChans), t_actions!([Chan1, Chan2, Chan3, Chan4], TogChans), t_actions!([Chan1, Chan2, Chan3, Chan4, Chan5], TogChans), t_actions!([Chan1, Chan2, Chan3, Chan4, Chan5, Chan3], TogChans)])]
    #[case(demo_mappings(), &[alt!(KeyA), key!(KeyJ), key!(KeyJ), key!(KeyK), key!(KeyK), key!(KeyK)], &[Timeout, ActionBeforeTimeout(VolDown), ActionBeforeTimeout(VolDown), ActionBeforeTimeout(VolUp), ActionBeforeTimeout(VolUp), ActionBeforeTimeout(VolUp)])]
    fn should_validate_demo_mappings(
        #[case] mappings: Vec<Vec<Mapping<TestAction, TestTag>>>,
        #[case] keypresses: &[KeyPress],
        #[case] expected: &[KeyHandlerAction<TestAction, TestTag>],
    ) {
        // Given
        let trie = MappingTrie::from_mappings(&mappings);
        let mut result = vec![];
        let mut buffers = Buffers::new();

        // When
        for key in keypresses {
            result.push(find_mapping(key, &trie, &mut buffers));
        }

        // Thre asd
        assert_eq!(result, expected)
    }
}
