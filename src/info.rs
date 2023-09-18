use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use scraper::{Html, Selector};
use std::collections::HashMap;

pub async fn info(user_agent: &str) -> HashMap<usize, Vec<(DateTime<Utc>, i64)>> {
    let document = get_webpage(user_agent)
        .await
        .expect("Can't reach info website.");

    // Selectors
    let sel_ul = Selector::parse("ul").unwrap();
    let sel_li = Selector::parse("li").unwrap();

    // Find the raw infos in html page
    let mut raw_data = Vec::new();
    for (i, data) in document.select(&sel_ul).enumerate() {
        if [1, 2].contains(&i) {
            raw_data.push(data);
        }
    }

    let mut data = HashMap::new();
    // d => date
    // r => repetition
    let re = Regex::new(r"(?P<d>\d{1,2} \w+ \d{4}).+(?P<r>\d)").unwrap();
    for (i, ul) in raw_data.into_iter().enumerate() {
        for element in ul.select(&sel_li) {
            match element.inner_html() {
                e if e.starts_with("Début") => {
                    let captures = re.captures(&e).unwrap();

                    let start_date = get_date(captures.name("d").unwrap().as_str());

                    let rep: i64 = captures.name("r").unwrap().as_str().parse().unwrap();

                    data.insert(i + 1, vec![(start_date, rep)]);
                }
                e if e.starts_with("Reprise") => {
                    let captures = re.captures(&e).unwrap();
                    captures.name("g");

                    let start_date = get_date(captures.name("d").unwrap().as_str());

                    let rep: i64 = captures.name("r").unwrap().as_str().parse().unwrap();

                    let it = i + 1;

                    let mut vec = data.get(&it).unwrap().to_owned();
                    vec.push((start_date, rep));

                    data.insert(it, vec);
                }
                _ => (),
            }
        }
    }

    data
}

/// Get info webpage
async fn get_webpage(user_agent: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let url = "https://informatique.up8.edu/licence-iv/edt";

    // Use custom User-Agent
    let client = reqwest::Client::builder().user_agent(user_agent).build()?;
    let html = client.get(url).send().await?.text().await?;

    // Panic on error
    crate::utils::check_errors(&html, url);

    Ok(Html::parse_document(&html))
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
