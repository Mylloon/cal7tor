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
