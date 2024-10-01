use std::collections::HashMap;

use chrono::Utc;

pub type InfoList = Vec<(chrono::DateTime<Utc>, i64)>;

pub struct InfoType {
    pub course: InfoList,
    pub td_tp: InfoList,
}

// Info who old the start and end of courses
pub type Info = HashMap<
    // Semester
    usize,
    // List of start and repetition of course and TD/TP weeks
    InfoType,
>;
