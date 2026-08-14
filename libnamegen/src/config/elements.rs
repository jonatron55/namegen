//! Element, attribute and namespace names shared by the XML parser and writer.

pub const NS_XSI: &str = "http://www.w3.org/2001/XMLSchema-instance";
pub const PREFIX_XSI: &str = "xsi";
pub const SCHEMA_LOCATION: &str = "namegen.xsd";

pub const ELEM_CAPITALIZE: &str = "Capitalize";
pub const ELEM_CASE: &str = "Case";
pub const ELEM_DEFAULT: &str = "Default";
pub const ELEM_DESCRIPTION: &str = "Description";
pub const ELEM_JOIN: &str = "Join";
pub const ELEM_LITERAL: &str = "Literal";
pub const ELEM_MARKOV: &str = "Markov";
pub const ELEM_MATCH: &str = "Match";
pub const ELEM_NUMBER: &str = "Number";
pub const ELEM_OPTION: &str = "Option";
pub const ELEM_PARAM: &str = "Param";
pub const ELEM_REJECT: &str = "Reject";
pub const ELEM_REPEAT: &str = "Repeat";
pub const ELEM_ROOT: &str = "NameGen";
pub const ELEM_SWITCH: &str = "Switch";
pub const ELEM_WORDS: &str = "Words";

pub const ELEM_SPLIT_TOKENIZER: &str = "SplitTokenizer";
pub const ELEM_CHUNK_TOKENIZER: &str = "ChunkTokenizer";
pub const ELEM_SSP_TOKENIZER: &str = "SspTokenizer";
pub const ELEM_CLASS: &str = "Class";

pub const ATTR_CUTOFF_LEN: &str = "cutoff_len";
pub const ATTR_DISPLAY_NAME: &str = "display_name";
pub const ATTR_EXPR: &str = "expr";
pub const ATTR_ID: &str = "id";
pub const ATTR_LEN: &str = "len";
pub const ATTR_MAX: &str = "max";
pub const ATTR_MIN: &str = "min";
pub const ATTR_MODE: &str = "mode";
pub const ATTR_PROBABILITY: &str = "probability";
pub const ATTR_RANK: &str = "rank";
pub const ATTR_REJECT_TRAINING: &str = "reject_training";
pub const ATTR_SCHEMA_LOCATION: &str = "noNamespaceSchemaLocation";
pub const ATTR_SEP: &str = "sep";
pub const ATTR_SPLIT_CHARS: &str = "split_chars";
pub const ATTR_STYLE: &str = "style";
pub const ATTR_TARGET_LEN: &str = "target_len";
pub const ATTR_TEXT: &str = "text";
pub const ATTR_UNIFORM: &str = "uniform";
