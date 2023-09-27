use chrono::{DateTime, Duration, Utc};
use regex::{Captures, Regex};
use scraper::Selector;
use std::collections::HashMap;

use crate::utils::{get_semester, get_webpage, get_year};

pub async fn info(
    level: i8,
    semester_opt: Option<i8>,
    year_opt: Option<i32>,
    user_agent: &str,
) -> HashMap<usize, Vec<(DateTime<Utc>, i64)>> {
    let semester = get_semester(semester_opt);

    let year = get_year(year_opt, semester);

    let document = get_webpage(level, semester, &year, user_agent)
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

    let re = Regex::new(r"\d{1,2} (septembre|octobre)").unwrap();
    let date = re.captures(&raw_data).unwrap().get(0).unwrap().as_str();

    let weeks_s1_1 = 6; // Number of weeks in the first part of the first semester
    let date_s1_1 = get_date(&format!("{} {}", date, year.split_once('-').unwrap().0)); // Get week of back-to-school
    let weeks_s1_2 = 7; // Number of weeks in the second part of the first semester
    let date_s1_2 = date_s1_1 + Duration::weeks(weeks_s1_1 + 1); // Add past weeks with the break-week

    let weeks_s2_1 = 6; // Number of weeks in the first part of the second semester
    let date_s2_1 = date_s1_2 + Duration::weeks(weeks_s1_2 + 4); // 4 weeks of vacation between semester
    let weeks_s2_2 = 7; // Number of weeks in the second part of the second semester
    let date_s2_2 = date_s2_1 + Duration::weeks(weeks_s2_1 + 1); // Add past weeks with the break-week

    HashMap::from([
        (
            1_usize,
            vec![(date_s1_1, weeks_s1_1), (date_s1_2, weeks_s1_2)],
        ),
        (
            2_usize,
            vec![(date_s2_1, weeks_s2_1), (date_s2_2, weeks_s2_2)],
        ),
    ])
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
        dico.keys().cloned().collect::<Vec<_>>().join("|")
    ))
    .unwrap();

    format!(
        // Use 12:00 and UTC TZ for chrono parser
        "{} 12:00 +0000",
        // Replace french by english month
        re.replace_all(date, |cap: &Captures| match &cap[0] {
            month if dico.contains_key(month) => dico.get(month).unwrap(),
            month => {
                panic!("Unknown month: {}", month)
            }
        })
    )
}

/// Turn a string to a DateTime
fn get_date(date: &str) -> DateTime<Utc> {
    // Use and keep UTC time, we have the hour set to 12h and
    // Paris 7 is in France so there is no problems
    DateTime::parse_from_str(&anglophonization(date), "%e %B %Y %H:%M %z")
        .unwrap()
        .into()
}
