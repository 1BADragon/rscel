use std::{collections::HashMap, sync::Arc};

use super::CelValue;

// Data type that moc's a CelValue but uses ref counters to store data. This is
// to prevent unnessesary copying of RO data, espessially if its large.
pub enum RefdCelValue {
    List(Vec<Arc<RefdCelValue>>),
    Map(HashMap<String, Arc<RefdCelValue>>),
    Value(CelValue),
}
