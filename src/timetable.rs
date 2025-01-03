#![allow(clippy::cast_sign_loss)]

use chrono::{Datelike, Duration, TimeZone, Utc};
use regex::Regex;
use scraper::Selector;
use std::{collections::HashMap, sync::Arc};

use crate::utils::{
    format_time_slot, get_hours, get_semester, get_webpage, get_year,
    models::{Info, InfoList},
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
                name: Regex::new(r"[ -][ML][1-3]$").unwrap().replace(
                    matches
                        .name("name")
                        .unwrap()
                        .as_str(),
                    ""
                ).to_string(),
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
        Some(&vec![models::Category::Cours]),
        None,
    );
    add_courses(
        &mut semester,
        &schedules,
        &timetable.1 .1,
        &datetimes.td_tp,
        None,
        Some(&vec![models::Category::Cours]),
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
    keep: Option<&Vec<models::Category>>,
    // List of category excluded
    exclude: Option<&Vec<models::Category>>,
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
                        .is_some_and(|list| !course.category.iter().any(|item| list.contains(item)))
                        || exclude.is_some_and(|list| {
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
pub fn display(timetable: &(Arc<[String]>, (usize, Vec<Day>))) {
    for day in &timetable.1 .1 {
        for (index, course_option) in day.courses.iter().enumerate() {
            if let Some(course) = course_option {
                if index == 0 {
                    println!("\n{}:", day.name);
                }

                println!(
                    "  {} - {} : {} ({}) // {}",
                    format_time_slot(course.start, course.size),
                    course
                        .category
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                    course.name,
                    course.room,
                    course.professor.as_deref().unwrap_or("N/A"),
                );
            }
        }
    }
}
