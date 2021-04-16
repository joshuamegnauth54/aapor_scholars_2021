use super::{conv_newtypes::*, query_structs::SteamRevOuter};
use crate::language::Language;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FlattenedQuery {

}

impl From<SteamRevOuter> for FlattenedQuery {
    fn from(other: SteamRevOuter) -> Self {
        unimplemented!()
    }
}
