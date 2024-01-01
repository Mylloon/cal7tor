use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Category {
    Cours,
    TP,
    TD,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Course {
    /// Type du cours
    pub category: Arc<[Category]>,

    /// Course's name
    pub name: String,

    /// Professor's name
    pub professor: Option<String>,

    /// List of rooms where the course takes place
    pub room: String,

    /// Time the course starts, as a number :
    /// - 0 => first possible class of the day
    /// - 1 => second possible class of the day
    /// - etc.
    pub start: usize,

    /// Number of time slots the course takes up in the timetable
    pub size: usize,

    /// Datetime when the course start
    /// Filled only when building for the ICS
    pub dtstart: Option<chrono::DateTime<chrono::Utc>>,

    /// Datetime when the course end
    /// Filled only when building for the ICS
    pub dtend: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct Day {
    /// Day's name
    pub name: String,
    /// Ordered list of all the courses of the day
    pub courses: Vec<Option<Course>>,
}

// Data builded in the timetable webpage
pub type Timetable = (
    // Schedules
    Arc<[String]>,
    // Timetable per days with the semester as the key
    (usize, Vec<Day>),
);
