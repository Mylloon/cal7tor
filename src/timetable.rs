use chrono::{Datelike, Duration, TimeZone, Utc};
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

use crate::utils::{
    self, capitalize,
    models::{Position, TabChar},
};

pub mod models;

/// Fetch the timetable for a class
pub async fn timetable(
    level: i8,
    semester_opt: Option<i8>,
    year_opt: Option<i32>,
    pathway: &str,
    user_agent: &str,
) -> (Vec<String>, (usize, Vec<models::Day>)) {
    let semester = get_semester(semester_opt);

    let year = get_year(year_opt, semester);

    let document = get_webpage(level, semester, &year, user_agent)
        .await
        .expect("Can't reach timetable website.");

    // Selectors
    let sel_table = Selector::parse("table").unwrap();
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_td = Selector::parse("td").unwrap();

    // Find the timetable
    let raw_timetable = document.select(&sel_table).next().unwrap();

    /* We are now iterating over all the 15-minute intervals to find courses */
    for element in raw_timetable
        .select(&sel_tbody)
        .next()
        .unwrap()
        .select(&sel_td)
    {
        if let Some(i) = element.value().attr("title") {
            println!("{}", i)
        }
    }

    todo!()
}

/// Get timetable webpage
async fn get_webpage(
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

/// Check if the timetable is well built
fn check_consistency(schedules: &[String], timetable: &Vec<models::Day>) -> bool {
    let mut checker = true;
    for day in timetable {
        let mut i = 0;
        for course in &day.courses {
            match course {
                Some(course_it) => {
                    // Checks the consistency of course start times
                    if i != course_it.start {
                        checker = false;
                        break;
                    }
                    // Keep the track of how many courses are in the day
                    i += course_it.size
                }
                None => i += 1,
            }
        }
        // The counter should be the same as the amount of possible hours of the day
        if i != schedules.len() {
            checker = false;
            break;
        }
    }

    checker
}

// Data builded in the timetable webpage
type T = (
    // Schedules
    Vec<String>,
    // Timetable per days with the semester as the key
    (usize, Vec<models::Day>),
);
// Data builded in the info webpage
type D = HashMap<
    // Semester
    usize,
    // List of start and repetition of course weeks
    Vec<(chrono::DateTime<Utc>, i64)>,
>;

/// Build the timetable
pub fn build(timetable: T, dates: D) -> Vec<models::Course> {
    let mut schedules = Vec::new();
    // h1 => heure de début | m1 => minute de début
    // h2 => heure de fin   | m2 => minute de fin
    let re =
        Regex::new(r"(?P<h1>\d{1,2})(h|:)(?P<m1>\d{1,2})?.(?P<h2>\d{1,2})(h|:)(?P<m2>\d{1,2})?")
            .unwrap();
    for hour in timetable.0 {
        let captures = re.captures(&hour).unwrap();

        let h1 = match captures.name("h1") {
            Some(h) => h.as_str().parse().unwrap(),
            None => 0,
        };
        let m1 = match captures.name("m1") {
            Some(h) => h.as_str().parse().unwrap(),
            None => 0,
        };
        let h2 = match captures.name("h2") {
            Some(h) => h.as_str().parse().unwrap(),
            None => 0,
        };
        let m2 = match captures.name("m2") {
            Some(h) => h.as_str().parse().unwrap(),
            None => 0,
        };
        schedules.push(((h1, m1), (h2, m2)));
    }

    // Store all the courses for the semester
    let mut semester = Vec::new();

    // Start date of the back-to-school week
    let datetimes = dates.get(&timetable.1 .0).unwrap();
    let before_break = datetimes.get(0).unwrap();
    let mut date = before_break.0;
    let mut rep = before_break.1;
    // For each weeks
    for _ in 0..2 {
        for _ in 0..rep {
            for day in &timetable.1 .1 {
                for mut course in day.courses.clone().into_iter().flatten() {
                    // Get the hours
                    let start = schedules.get(course.start).unwrap().0;
                    // -1 because we only add when the size is > 1
                    let end = schedules.get(course.start + course.size - 1).unwrap().1;

                    // Add the changed datetimes
                    course.dtstart = Some(
                        Utc.with_ymd_and_hms(
                            date.year(),
                            date.month(),
                            date.day(),
                            start.0,
                            start.1,
                            0,
                        )
                        .unwrap(),
                    );
                    course.dtend = Some(
                        Utc.with_ymd_and_hms(
                            date.year(),
                            date.month(),
                            date.day(),
                            end.0,
                            end.1,
                            0,
                        )
                        .unwrap(),
                    );

                    semester.push(course);
                }
                date += Duration::days(1);
            }
            // From friday to monday
            date += Duration::days(2);
        }
        let after_break = datetimes.get(1).unwrap();
        date = after_break.0;
        rep = after_break.1;
    }

    semester
}

/// Get the current semester depending on the current date
fn get_semester(semester: Option<i8>) -> i8 {
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
fn get_year(year: Option<i32>, semester: i8) -> String {
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

/// Display the timetable
pub fn display(timetable: (Vec<String>, (usize, Vec<models::Day>)), cell_length: usize) {
    // Cell length for hours
    let clh = 11;
    // Cell number
    let cn = 6;
    // 3/4 of cell length
    let quarter = (3 * cell_length) / 4;

    let sep = TabChar::Bv.val();

    // Top of the tab
    utils::line_table(clh, cell_length, cn, Position::Top, HashMap::new());

    // First empty case
    print!("{}{:^clh$}{}", sep, "", sep);

    // Print day's of the week
    let mut days = HashMap::new();
    for (i, data) in timetable.1 .1.iter().enumerate() {
        days.insert(i, &data.name);
        print!("{:^cell_length$}{}", &data.name, sep);
    }

    // Store the data of the course for utils::line_table
    let mut next_skip = HashMap::new();
    // For each hours -- i the hour's number
    for (i, hour) in timetable.0.into_iter().enumerate() {
        // Draw separator line
        utils::line_table(clh, cell_length, cn, Position::Middle, next_skip);

        // Reset
        next_skip = HashMap::new();

        // Print hour
        print!("{}{:^clh$}", sep, hour);

        // For all the days - `j` the day's number
        for (j, day) in timetable.1 .1.iter().enumerate() {
            // True if we found something about the slot we are looking for
            let mut info_slot = false;

            // For all the courses of each days - `k` the possible course.start
            for (k, course_opt) in day.courses.iter().enumerate() {
                match course_opt {
                    // If there is a course
                    Some(course) => {
                        // Check the course's hour
                        if i == course.start {
                            // If the course uses more than one time slot
                            if course.size > 1 {
                                // If the data is too long
                                if course.name.len() > quarter {
                                    let data = utils::split_half(&course.name);
                                    next_skip.insert(j, data.1.trim());
                                    print!("{}{:^cell_length$}", sep, data.0.trim());
                                } else {
                                    next_skip.insert(j, &course.name);
                                    print!("{}{:^cell_length$}", sep, "");
                                }
                                info_slot = true;
                                break;
                            } else {
                                // Else simply print the course
                                // If the data is too long
                                if course.name.len() > quarter {
                                    print!("{}{:^cell_length$}", sep, utils::etc_str(&course.name));
                                } else {
                                    print!("{}{:^cell_length$}", sep, &course.name);
                                }
                                info_slot = true;
                                break;
                            }
                        }
                    }
                    // If no course was found
                    None => {
                        // Verify the "no course" is in the correct day and hour
                        if *days.get(&j).unwrap() == &day.name.to_string() && k == i {
                            // If yes print empty row because there is no course
                            print!("{}{:^cell_length$}", sep, "");
                            info_slot = true;
                            break;
                        }
                        // Else it was a course of another day/time
                    }
                };
            }
            if !info_slot {
                // We found nothing about the slot because the precedent course
                // takes more place than one slot
                print!("{}{:^cell_length$}", sep, "");
            }
        }
        print!("{}", sep);
    }
    // Bottom of the table
    utils::line_table(clh, cell_length, cn, Position::Bottom, HashMap::new());
}
