use crate::block::SpannedBlock;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockOperation {
    Let {
        // #[serde(rename = "$let")]
        variables: SpannedBlock,
        // #[serde(rename = "in")]
        body: SpannedBlock,
    },
    If {
        // #[serde(rename = "$if")]
        condition: SpannedBlock,
        // #[serde(rename = "then")]
        consequent: Option<SpannedBlock>,
        // #[serde(rename = "else")]
        alternative: Option<SpannedBlock>,
    },
}
