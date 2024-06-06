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
pub struct ActionsOnTimeout<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    pub actions: Vec<A>,
    pub tag: Option<T>,
}

impl<A, T> ActionsOnTimeout<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    pub fn new() -> Self {
        Self {
            actions: vec![],
            tag: None,
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

impl<A, T> Display for ActionsOnTimeout<A, T>
where
    A: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
    T: PartialEq + Eq + Clone + Debug + Display + Send + Sync + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ActionsOnTimeout(actions: {:?}, tag: {:?})",
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
    actions: &mut ActionsOnTimeout<A, T>,
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
        },
        Choice(behaviours, tag) => {
            match behaviours.get_mapping(key) {
                Some(mapping) => match (mapping, actions.get_tag()) {
                    (Behaviour::Timeout(_), None) => Timeout,
                    (Behaviour::Timeout(_), Some(_)) => ActionsOnTimeout(actions.clone()),
                    (Behaviour::Action(_, action), Some(_)) => ActionBeforeTimeout(action.clone()),
                    (Behaviour::Action(_, action), None) => ActionBeforeAndOnTimeout {
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
                },
                None => Nothing, // Should never happen.
            }
        }
    }
}
