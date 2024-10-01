use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{Datelike, Utc};
use scraper::Html;

use crate::timetable::models::{Category, Course, Timetable};

pub mod models;

/// Panic if an error happened
pub fn check_errors(html: &String, loc: &str) {
    let no_timetable = "Aucun créneau horaire affecté";
    match html {
        t if t.contains(no_timetable) => panic!("URL: {loc} • {no_timetable}"),
        _ => (),
    }
}

/// Get timetable webpage
pub async fn get_webpage(
    level: i8,
    semester: i8,
    year: &str,
    user_agent: &str,
) -> Result<Html, Box<dyn std::error::Error>> {
    let url = format!("https://silice.informatique.univ-paris-diderot.fr/ufr/U{year}/EDT/visualiserEmploiDuTemps.php?quoi=M{level},{semester}");

    // Use custom User-Agent
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let html = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await?
        .text()
        .await?;

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

/// Get all hours used the source, from 08:00 to at least 20:00
pub fn get_hours() -> Arc<[String]> {
    let mut hours = vec![];
    for hour in 8..=20 {
        for minute in &[0, 15, 30, 45] {
            let hour_str = format!("{hour}h{minute:02}");
            if let Some(last_hour) = hours.pop() {
                hours.push(format!("{last_hour}-{hour_str}"));
            }
            hours.push(hour_str);
        }
    }
    for _ in 0..4 {
        hours.pop();
    }

    hours.into()
}

/// Names showed to the users
pub fn get_selection(data: &(&Course, String)) -> String {
    let hours = get_hours();

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

/// Entry's name used for finding duplicates, ignoring categories
pub fn get_entry_nocat(course: &Course) -> String {
    course.name.clone()
}

/// Returns a couple of (list of courses) and (a hashmap of how much they appears in the vector)
pub fn get_count<'a>(
    timetable: &'a mut Timetable,
    allowed_list: &'a [Category],
    getter: fn(&Course) -> String,
) -> (Vec<(&'a Course, String)>, HashMap<String, i32>) {
    // List of courses who will be courses
    let mut courses = vec![];

    let mut counts = HashMap::new();
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                if course
                    .category
                    .iter()
                    .any(|category| allowed_list.contains(category))
                {
                    courses.push((course, day.name.clone()));
                    let count = counts.entry(getter(course)).or_insert(0);
                    *count += 1;
                }
            }
        });
    });

    (courses, counts)
}

pub fn format_time_slot(start: usize, size: usize) -> String {
    let start_hour = 8 + (start * 15) / 60;
    let start_minute = (start * 15) % 60;
    let end_hour = start_hour + (size * 15) / 60;
    let end_minute = (start_minute + (size * 15)) % 60;

    format!("{start_hour:02}h{start_minute:02}-{end_hour:02}h{end_minute:02}")
}
