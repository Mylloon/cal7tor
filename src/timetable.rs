#![allow(clippy::cast_sign_loss)]

use chrono::{Datelike, Duration, TimeZone, Utc};
use regex::Regex;
use scraper::Selector;
use std::{collections::HashMap, sync::Arc};

use crate::utils::{
    self, get_hours, get_semester, get_webpage, get_year,
    models::{Info, InfoList, Position, TabChar},
    Capitalize,
};

use self::models::Day;

pub mod models;

/// Fetch the timetable for a class
pub async fn timetable(
    level: i8,
    semester_opt: Option<i8>,
    year_opt: Option<i32>,
    user_agent: &str,
) -> models::Timetable {
    let semester = get_semester(semester_opt);

    let year = get_year(year_opt, semester);

    let document = get_webpage(level, semester, &year, user_agent)
        .await
        .expect("Can't reach timetable website.");

    // Selectors
    let sel_table = Selector::parse("table").unwrap();
    let sel_tbody = Selector::parse("tbody").unwrap();
    let sel_td = Selector::parse("td").unwrap();
    let sel_small = Selector::parse("small").unwrap();
    let sel_b = Selector::parse("b").unwrap();
    let sel_span = Selector::parse("span").unwrap();

    // Find the timetable
    let raw_timetable = document.select(&sel_table).next().unwrap();

    let schedules = get_hours();

    let mut timetable: Vec<models::Day> = Vec::new();

    raw_timetable
        .select(&sel_tbody)
        .next()
        .unwrap()
        .select(&sel_td)
        .filter(|element| element.value().attr("title").is_some())
        .for_each(|i| {
            let extra_data = i.select(&sel_span).next().map(|span|
                 span.inner_html().replace("<br>", "").trim().to_owned()
            );

            /* TODO: Instead of searching *_M2, just find any TD_* and TP_* */
            let matches =
                Regex::new(
                    r"(?P<type>COURS|COURS_TD|TD|TD_M2|TP|TP_M2)? (?P<name>.*) : (?P<day>(lundi|mardi|mercredi|jeudi|vendredi)) (?P<startime>.*) \(durée : (?P<duration>.*)\)")
                    .unwrap()
                    .captures(i.value().attr("title").unwrap())
                    .unwrap();

            let day = matches
                .name("day")
                .unwrap()
                .as_str()
                .capitalize();

            let startime = matches
            .name("startime")
            .unwrap()
            .as_str();

            let binding = i.select(&sel_b).last().unwrap().inner_html();
            let course = models::Course{
                category: match matches
                .name("type")
                .map_or("", |m| m.as_str()) {
                    /* TODO: Instead of searching *_M2, just find any TD_* and TP_* */
                    "COURS" => [models::Category::Cours].into(),
                    "TP" | "TP_M2" => [models::Category::TP].into(),
                    "TD" | "TD_M2" => [models::Category::TD].into(),
                    "COURS_TD" => [models::Category::Cours, models::Category::TD].into(),
                    _ => {
                        println!("Unknown type of course, falling back to 'COURS': {}", i.value().attr("title").unwrap());
                        [models::Category::Cours].into()
                    },
                },
                name: matches
                .name("name")
                .unwrap()
                .as_str().to_owned(),
                professor: if let Some(raw_prof) = i.select(&sel_small).last() {
                    match raw_prof.inner_html() {
                        i if i.starts_with("<span") => None,
                        i => Some(i),
                    }
                } else { None },
                room: Regex::new(r"(<table.*<\/table>|<br>.*?<br>.*?)?<br>(?P<location>.*?)<br>")
                .unwrap()
                .captures(&binding)
                .unwrap().name("location")
                .unwrap()
                .as_str().to_owned(),
                start: schedules.iter().position(|r| r.starts_with(startime)).unwrap(),
                size: i.value().attr("rowspan").unwrap().parse::<usize>().unwrap(),
                dtstart: None,
                dtend: None,
                data: extra_data,
            };

            // Search for the day in the timetable
            if let Some(existing_day) = timetable.iter_mut().find(|x| x.name == day) {
                existing_day.courses.push(Some(course));
            } else {
                // Day with the name doesn't exist, create a new Day
                timetable.push(models::Day {
                    name: day.clone(),
                    courses: vec![Some(course)],
                });
            }
        });

    // Sort by days
    let day_positions = ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi"]
        .iter()
        .enumerate()
        .map(|(i, &day)| (day.to_owned(), i))
        .collect::<HashMap<String, usize>>();
    timetable.sort_by(|a, b| day_positions[&a.name].cmp(&day_positions[&b.name]));

    (schedules, (semester as usize, timetable))
}

/// Build the timetable
pub fn build(timetable: &models::Timetable, dates: &Info) -> Vec<models::Course> {
    let mut schedules = Vec::new();
    // h1 => heure de début | m1 => minute de début
    // h2 => heure de fin   | m2 => minute de fin
    let re = Regex::new(r"(?P<h1>\d{1,2})h(?P<m1>\d{2})-(?P<h2>\d{1,2})h(?P<m2>\d{2})").unwrap();
    for hour in timetable.0.iter() {
        let captures = re.captures(hour).unwrap();

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
    add_courses(
        &mut semester,
        &schedules,
        &timetable.1 .1,
        &datetimes.course,
        &Some(vec![models::Category::Cours]),
        &None,
    );
    add_courses(
        &mut semester,
        &schedules,
        &timetable.1 .1,
        &datetimes.td_tp,
        &None,
        &Some(vec![models::Category::Cours]),
    );

    semester
}

type Schedule = [((u32, u32), (u32, u32))];

/// Add a course to the semester list
fn add_courses(
    // Accumulator of courses of semester
    semester: &mut Vec<models::Course>,
    // Hours used
    schedules: &Schedule,
    // List of days
    days: &Vec<Day>,
    // Current courses list
    info: &InfoList,
    // List of category allowed
    keep: &Option<Vec<models::Category>>,
    // List of category excluded
    exclude: &Option<Vec<models::Category>>,
) {
    let before_break = info.first().unwrap();
    let mut date = before_break.0;
    let mut rep = before_break.1;
    // For each weeks
    for _ in 0..2 {
        for _ in 0..rep {
            for day in days {
                for mut course in day.courses.iter().flatten().cloned() {
                    // Get the hours
                    let start = schedules.get(course.start).unwrap().0;
                    // -1 because we only add when the size is > 1
                    let end = schedules.get(course.start + course.size - 1).unwrap().1;

                    // Check keep and exclude filters
                    if keep
                        .clone()
                        .is_some_and(|list| !course.category.iter().any(|item| list.contains(item)))
                        || exclude.clone().is_some_and(|list| {
                            course.category.iter().any(|item| list.contains(item))
                        })
                    {
                        continue;
                    }

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
        let after_break = info.last().unwrap();
        date = after_break.0;
        rep = after_break.1;
    }
}

/// Display the timetable
pub fn display(timetable: &(Arc<[String]>, (usize, Vec<models::Day>)), cell_length: usize) {
    // Cell length for hours
    let clh = 11;
    // Cell number
    let cn = 6;
    // 3/4 of cell length
    let quarter = (3 * cell_length) / 4;

    let sep = TabChar::Bv.val();

    // Top of the tab
    utils::line_table(clh, cell_length, cn, &Position::Top, &HashMap::new());

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
    for (i, hour) in timetable.0.iter().enumerate() {
        // Draw separator line
        utils::line_table(clh, cell_length, cn, &Position::Middle, &next_skip);

        // Reset
        next_skip = HashMap::new();

        // Print hour
        print!("{sep}{hour:^clh$}");

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
                            }

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
        print!("{sep}");
    }
    // Bottom of the table
    utils::line_table(clh, cell_length, cn, &Position::Bottom, &HashMap::new());
}
