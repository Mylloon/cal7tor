#[derive(Clone, Debug)]
pub enum Type {
    Cours,
    TP,
    TD,
}

#[derive(Clone, Debug)]
pub struct Course {
    /// Type du cours
    pub typee: Type,

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
