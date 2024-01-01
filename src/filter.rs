use std::collections::HashMap;

use dialoguer::MultiSelect;

use crate::timetable::models::Category;
use crate::timetable::models::Course;
use crate::timetable::models::Timetable;
use crate::utils::fill_hours;

const DISCLAIMER: &str = "(selection avec ESPACE, ENTRER pour valider)";

/// Filter the timetable
pub fn timetable(timetable: Timetable) -> Timetable {
    let mut my_timetable = timetable;

    courses(&mut my_timetable);
    tdtp(&mut my_timetable);

    my_timetable
}

/// Exclude some courses
fn courses(timetable: &mut Timetable) {
    let mut multiselected = vec![];
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                if !multiselected.contains(&course.name) {
                    multiselected.push(course.name.to_owned());
                }
            }
        })
    });

    let defaults = vec![true; multiselected.len()];
    let selections = MultiSelect::new()
        .with_prompt(format!("Choisis tes matières {}", DISCLAIMER))
        .items(&multiselected[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    for day in &mut timetable.1 .1 {
        day.courses.retain(|course_opt| {
            if let Some(course) = course_opt {
                // Remove courses not followed
                for i in &selections {
                    if course.name == multiselected[*i] {
                        return true;
                    }
                }
            }

            false
        });
    }
}

/// Filter the multiples TD/TP
fn tdtp(timetable: &mut Timetable) {
    // Entry's name used for finding duplicates
    let get_entry = |course: &Course| format!("{} - {:?}", course.name, course.category);

    let mut hours = vec![];
    fill_hours(&mut hours);

    // Names showed to the users
    let get_selection = |data: &(&Course, String)| {
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
    };

    // List of courses who will be TP/TD
    let mut td_or_tp = vec![];

    // Counter of appearing of TP/TD to know if a TP/TD have multiple possible course
    let mut counts = HashMap::new();
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                match course.category {
                    Category::TD | Category::TP => {
                        td_or_tp.push((course, day.name.to_owned()));
                        let count = counts.entry(get_entry(course)).or_insert(0);
                        *count += 1;
                    }
                    _ => (),
                }
            }
        })
    });

    // Keep only elements who have multiples TD/TP
    td_or_tp.retain(|course| *counts.get(&get_entry(course.0)).unwrap() > 1);

    let mut multiselected: Vec<String> = td_or_tp.iter().map(get_selection).collect();
    multiselected.sort();

    let defaults = vec![false; multiselected.len()];
    let selections = MultiSelect::new()
        .with_prompt(format!("Choisis tes horaires de TD/TP {}", DISCLAIMER))
        .items(&multiselected[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    // Keep only wanted courses
    for day in &mut timetable.1 .1 {
        day.courses.retain(|course_opt| {
            if let Some(course) = course_opt {
                // Keep if it's a course
                if course.category == Category::Cours {
                    return true;
                }

                // Keep if its an -only one course- TD/TP
                if *counts.get(&get_entry(course)).unwrap() == 1 {
                    return true;
                }

                // Remove courses not followed
                for i in &selections {
                    if get_selection(&(course, day.name.to_owned())) == multiselected[*i] {
                        return true;
                    }
                }
            }

            false
        });
    }
}
