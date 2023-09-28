use std::collections::HashMap;

use dialoguer::MultiSelect;

use crate::timetable::models::Course;
use crate::timetable::models::Timetable;
use crate::timetable::models::Type;

const DISCLAIMER: &str = "(selection avec ESPACE, ENTRER pour valider)";

/// Filter the timetable
pub fn timetable(timetable: Timetable) -> Timetable {
    let mut my_timetable = timetable;

    /* courses(&mut my_timetable); */
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
    let get_entry = |course: &Course| format!("{} - {:?}", course.name, course.typee);
    let mut td_or_tp = vec![];

    let mut counts = HashMap::new();
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                match course.typee {
                    Type::TD | Type::TP => {
                        td_or_tp.push(course);
                        let count = counts.entry(get_entry(course)).or_insert(0);
                        *count += 1;
                    }
                    _ => (),
                }
            }
        })
    });
    // Keep only elements who have multiples TD/TP
    td_or_tp.retain(|&course| *counts.get(&get_entry(course)).unwrap() > 1);

    let multiselected = td_or_tp
        .iter()
        .map(|el| format!("{} - {}", el.name, el.size))
        .collect::<Vec<String>>();

    let defaults = vec![true; multiselected.len()];
    let selections = MultiSelect::new()
        .with_prompt(format!("Choisis tes horaires de TD/TP {}", DISCLAIMER))
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
