use chrono::{DateTime, Duration, Utc};
use regex::{Captures, Regex};
use scraper::Selector;
use std::{collections::HashMap, sync::Arc};

use crate::utils::{
    get_period_weeks, get_semester, get_webpage, get_year,
    models::{Info, InfoList, InfoType},
};

pub async fn get_start_date(
    level: i8,
    semester_opt: Option<i8>,
    year_opt: Option<i32>,
    user_agent: &str,
) -> String {
    let semester = get_semester(semester_opt);
    let year = get_year(year_opt, semester);

    // Fetch the timetable of the FIRST semester
    let document = get_webpage(level, 1, &year, user_agent)
        .await
        .expect("Can't reach info website.");

    // Selectors
    let sel_b = Selector::parse("b").unwrap();
    let sel_font = Selector::parse("font").unwrap();

    // Find when is the back-to-school date
    let raw_data = document
        .select(&sel_b)
        .find(|element| element.select(&sel_font).next().is_some())
        .unwrap()
        .inner_html();

    let first_semester = semester == 1;

    let re = Regex::new(if first_semester {
        r"\d{1,2} (septembre|octobre)"
    } else {
        r"\d{1,2} (janvier|février)"
    })
    .unwrap();

    re.captures(&raw_data).and_then(|caps| caps.get(0)).map_or(
        if first_semester {
            "1 septembre"
        } else {
            "1 janvier"
        }
        .to_owned(),
        |m| m.as_str().to_owned(),
    )
}

#[allow(clippy::bool_to_int_with_if)]
pub fn info(
    semester_opt: Option<i8>,
    year_opt: Option<i32>,
    date: &str,
    skip_week: bool,
    holidays: bool,
    weeks: Option<i32>,
) -> Info {
    let semester = get_semester(semester_opt);
    let year = get_year(year_opt, semester);

    // 1st semester
    let (weeks_s1_1, weeks_s1_2) = get_period_weeks(1, weeks); // Weeks before and after break
    let date_s1_1 = get_date(&format!("{} {}", date, year.split_once('-').unwrap().0)); // Get first week of school
    let date_s1_2 = date_s1_1 + Duration::weeks(weeks_s1_1 + if holidays { 1 } else { 0 }); // Back-to-school week

    // 2nd semester
    let (weeks_s2_1, weeks_s2_2) = get_period_weeks(2, weeks); // Weeks before and after break
    let date_s2_1 = get_date(&format!("{} {}", date, year.split_once('-').unwrap().1)); // Get first week
    let date_s2_2 = date_s2_1 + Duration::weeks(weeks_s2_1 + if holidays { 2 } else { 0 }); // Back-to-school week

    // Group courses values and derive it for TD/TP
    let cours_s1 = vec![(date_s1_1, weeks_s1_1), (date_s1_2, weeks_s1_2)];
    let cours_s2 = vec![(date_s2_1, weeks_s2_1), (date_s2_2, weeks_s2_2)];

    let delta = i64::from(skip_week);

    let tdtp_s1 = derive_from_cours(&cours_s1, delta);
    let tdtp_s2 = derive_from_cours(&cours_s2, delta);

    HashMap::from([
        (
            1_usize,
            InfoType {
                course: cours_s1,
                td_tp: tdtp_s1,
            },
        ),
        (
            2_usize,
            InfoType {
                course: cours_s2,
                td_tp: tdtp_s2,
            },
        ),
    ])
}

/// Find TD/TP dates, based on the ones from courses
fn derive_from_cours(courses: &InfoList, delta: i64) -> Vec<(DateTime<Utc>, i64)> {
    // TD/TP start one week after courses
    let before_break = courses.first().unwrap();
    let after_break = courses.last().unwrap();
    vec![
        (
            before_break.0 + Duration::weeks(delta),
            before_break.1 - delta,
        ),
        (after_break.0, after_break.1 + delta),
    ]
}

/// Turn a french date to an english one
fn anglophonization(date: &str) -> String {
    let dico = HashMap::from([
        ("janvier", "january"),
        ("février", "february"),
        ("mars", "march"),
        ("avril", "april"),
        ("mai", "may"),
        ("juin", "june"),
        ("juillet", "july"),
        ("août", "august"),
        ("septembre", "september"),
        ("octobre", "october"),
        ("novembre", "november"),
        ("décembre", "december"),
    ]);

    // New regex of all the french month
    let re = Regex::new(&format!(
        "({})",
        dico.keys().copied().collect::<Arc<[_]>>().join("|")
    ))
    .unwrap();

    format!(
        // Use 12:00 and UTC TZ for chrono parser
        "{} 12:00 +0000",
        // Replace french by english month
        re.replace_all(&date.to_lowercase(), |cap: &Captures| match &cap[0] {
            month if dico.contains_key(month) => dico.get(month).unwrap(),
            month => {
                panic!("Unknown month: {month}")
            }
        })
    )
}

/// Turn a string to a `DateTime`
fn get_date(date: &str) -> DateTime<Utc> {
    // Use and keep UTC time, we have the hour set to 12h and
    // Paris 7 is in France so there is no problems
    DateTime::parse_from_str(&anglophonization(date), "%e %B %Y %H:%M %z")
        .unwrap()
        .into()
}
