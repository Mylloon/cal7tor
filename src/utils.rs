use std::collections::HashMap;

use chrono::{Datelike, Utc};
use scraper::Html;

use crate::timetable::models::{Category, Course, Timetable};

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

/// Get timetable webpage
pub async fn get_webpage(
    level: i8,
    semester: i8,
    year: &str,
    user_agent: &str,
) -> Result<Html, Box<dyn std::error::Error>> {
    let url = format!("https://silice.informatique.univ-paris-diderot.fr/ufr/U{}/EDT/visualiserEmploiDuTemps.php?quoi=M{},{}", year, level, semester);

    // Use custom User-Agent
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let html = client.get(&url).send().await?.text().await?;

    // Panic on error
    crate::utils::check_errors(&html, &url);

    // Parse document
    let document = Html::parse_document(&html);

    Ok(document)
}

/// Get the current semester depending on the current date
pub fn get_semester(semester: Option<i8>) -> i8 {
    match semester {
        // Force the asked semester
        Some(n) => n,
        // Find the potential semester
        None => {
            if Utc::now().month() > 6 {
                // From july to december
                1
            } else {
                // from january to june
                2
            }
        }
    }
}

/// Get the current year depending on the current date
pub fn get_year(year: Option<i32>, semester: i8) -> String {
    let wanted_year = match year {
        // Force the asked semester
        Some(n) => n,
        // Find the potential semester
        None => Utc::now().year(),
    };

    if semester == 1 {
        format!("{}-{}", wanted_year, wanted_year + 1)
    } else {
        format!("{}-{}", wanted_year - 1, wanted_year)
    }
}

pub trait Capitalize {
    /// Capitalize string
    fn capitalize(&self) -> String;
}

impl Capitalize for str {
    fn capitalize(&self) -> String {
        let mut string = self.to_owned();
        if let Some(r) = string.get_mut(0..1) {
            r.make_ascii_uppercase();
        }

        string
    }
}

pub fn fill_hours(hours: &mut Vec<String>) {
    for hour in 8..=20 {
        for minute in &[0, 15, 30, 45] {
            let hour_str = format!("{}h{:02}", hour, minute);
            if let Some(last_hour) = hours.pop() {
                hours.push(format!("{}-{}", last_hour, hour_str));
            }
            hours.push(hour_str);
        }
    }
    for _ in 0..4 {
        hours.pop();
    }
}

/// Names showed to the users
pub fn get_selection(data: &(&Course, String)) -> String {
    let mut hours = vec![];
    fill_hours(&mut hours);

    format!(
        "{} - {} {}-{}",
        data.0.name,
        data.1,
        hours[data.0.start].split_once('-').unwrap().0,
        hours[data.0.start + data.0.size - 1]
            .split_once('-')
            .unwrap()
            .1
    )
}

/// Entry's name used for finding duplicates
pub fn get_entry(course: &Course) -> String {
    format!("{} - {:?}", course.name, course.category)
}

pub fn get_count<'a>(
    timetable: &'a mut Timetable,
    allowed_list: &'a [Category],
) -> (Vec<(&'a Course, String)>, HashMap<String, i32>) {
    // List of courses who will be courses
    let mut courses = vec![];

    let mut counts = HashMap::new();
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                if allowed_list.contains(&course.category) {
                    courses.push((course, day.name.to_owned()));
                    let count = counts.entry(get_entry(course)).or_insert(0);
                    *count += 1;
                }
            }
        })
    });

    (courses, counts)
}
