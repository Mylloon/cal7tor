pub mod models;

/// Panic if an error happened
pub fn check_errors(html: &String, loc: &str) {
    let no_timetable = "Aucun créneau horaire affecté";
    match html {
        t if t.contains(no_timetable) => panic!("URL: {} • {}", loc, no_timetable),
        _ => (),
    }
}

/// Print a line for the table
pub fn line_table(
    cell_length_hours: usize,
    cell_length: usize,
    number_cell: usize,
    pos: models::Position,
    skip_with: std::collections::HashMap<usize, &str>,
) {
    // Left side
    let ls = match pos {
        models::Position::Top => models::TabChar::Jtl.val(),
        models::Position::Middle => models::TabChar::Jl.val(),
        models::Position::Bottom => models::TabChar::Jbl.val(),
    };

    // Middle
    let ms = match pos {
        models::Position::Top => models::TabChar::Jtb.val(),
        models::Position::Middle => models::TabChar::Jm.val(),
        models::Position::Bottom => models::TabChar::Jtt.val(),
    };

    // Right side
    let rs = match pos {
        models::Position::Top => models::TabChar::Jtr.val(),
        models::Position::Middle => models::TabChar::Jr.val(),
        models::Position::Bottom => models::TabChar::Jbr.val(),
    };

    // Right side before big cell
    let rs_bbc = models::TabChar::Jr.val();
    // Right side big cell before big cell
    let rsbc_bbc = models::TabChar::Bv.val();
    // Right side big cell
    let rsbc = models::TabChar::Jl.val();

    let line = models::TabChar::Bh.val().to_string().repeat(cell_length);
    let line_h = models::TabChar::Bh
        .val()
        .to_string()
        .repeat(cell_length_hours);

    // Hours column
    match skip_with.get(&0) {
        Some(_) => print!("\n{}{}{}", ls, line_h, rs_bbc),
        None => print!("\n{}{}{}", ls, line_h, ms),
    };

    // Courses columns
    let range = number_cell - 1;
    let mut last_day = false;
    for i in 0..range {
        // Check if it's a big cell
        if i == range - 1 {
            // Friday only
            if let Some(text) = skip_with.get(&i) {
                println!("{:^cell_length$}{}", text, rsbc_bbc);
                last_day = true;
            }
        } else {
            match skip_with.get(&i) {
                Some(text) => match skip_with.get(&(i + 1)) {
                    // Match check if the next cell will be big
                    Some(_) => print!("{:^cell_length$}{}", text, rsbc_bbc),
                    None => print!("{:^cell_length$}{}", text, rsbc),
                },
                None => match skip_with.get(&(i + 1)) {
                    // Match check if the next cell will be big
                    Some(_) => print!("{}{}", line, rs_bbc),
                    None => print!("{}{}", line, ms),
                },
            }
        }
    }
    if !last_day {
        println!("{}{}", line, rs);
    }
}

// Split a string in half with respect of words
pub fn split_half(text: &str) -> (&str, &str) {
    let mid = text.len() / 2;
    for (i, j) in (mid..text.len()).enumerate() {
        if text.as_bytes()[j] == b' ' {
            return text.split_at(mid + i);
        }
    }

    text.split_at(mid)
}

// Reduce size of string by adding etc. to it, and cutting some info
pub fn etc_str(text: &str) -> String {
    format!("{}...", split_half(text).0.trim())
}

// Capitalize string
pub fn capitalize(text: &mut str) -> String {
    if let Some(r) = text.get_mut(0..1) {
        r.make_ascii_uppercase();
    }

    text.to_string()
}
