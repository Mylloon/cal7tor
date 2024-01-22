use std::collections::HashMap;

use chrono::Utc;

/// Collection of char for the table
pub enum TabChar {
    /// Vertical bar
    Bv,
    /// Horizontal bar
    Bh,
    /// Joint left
    Jl,
    /// Joint right
    Jr,
    /// Joint bottom left
    Jbl,
    /// Joint bottom right
    Jbr,
    /// Joint top left
    Jtl,
    /// Joint top right
    Jtr,
    /// Joint to top
    Jtt,
    /// Joint to bottom
    Jtb,
    /// Joint of the middle
    Jm,
}

impl TabChar {
    /// Value of the element
    pub fn val(&self) -> char {
        match *self {
            Self::Bv => '│',
            Self::Bh => '─',
            Self::Jl => '├',
            Self::Jr => '┤',
            Self::Jbl => '└',
            Self::Jbr => '┘',
            Self::Jtl => '┌',
            Self::Jtr => '┐',
            Self::Jtt => '┴',
            Self::Jtb => '┬',
            Self::Jm => '┼',
        }
    }
}

/// Position for lines inside the table
pub enum Position {
    Top,
    Middle,
    Bottom,
}

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
