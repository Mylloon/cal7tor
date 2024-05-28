use dialoguer::MultiSelect;

use crate::timetable::models::Category;
use crate::timetable::models::Timetable;
use crate::utils::get_count;
use crate::utils::get_entry;
use crate::utils::get_entry_nocat;
use crate::utils::get_selection;

const DISCLAIMER: &str = "(selection avec ESPACE, ENTRER pour valider)";

/// Filter the timetable
pub fn timetable(timetable: Timetable, merge_td_tp: bool) -> Timetable {
    let mut my_timetable = timetable;

    /* Note on Cours/TD:
     * We use the "as long as x interests us, we accept" approach.
     *
     * Because when a course and its TD are on the same slot,
     * it's probably because there's an alternation between course
     * and TD and no other choice is possible. */

    choice(&mut my_timetable);
    courses(&mut my_timetable);
    tdtp(&mut my_timetable, merge_td_tp);

    my_timetable
}

/// Exclude some courses
fn choice(timetable: &mut Timetable) {
    let mut multiselected = vec![];
    timetable.1 .1.iter().for_each(|day| {
        day.courses.iter().for_each(|course_opt| {
            if let Some(course) = course_opt {
                if !multiselected.contains(&course.name) {
                    multiselected.push(course.name.clone());
                }
            }
        });
    });

    let defaults = vec![true; multiselected.len()];
    let selections = MultiSelect::new()
        .with_prompt(format!("Choisis tes matières {DISCLAIMER}"))
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

/// Filter the multiple courses
fn courses(timetable: &mut Timetable) {
    let entry_getter = get_entry;

    // List of courses and Counter of how much they appears
    // to know if multiples slots are available
    let (mut courses, counts) = get_count(timetable, &[Category::Cours], entry_getter);

    // Keep only elements who have multiples slots
    courses.retain(|course| *counts.get(&entry_getter(course.0)).unwrap() > 1);

    let mut multiselected: Vec<String> = courses.iter().map(get_selection).collect();
    multiselected.sort();

    let mut selections = vec![];
    if !multiselected.is_empty() {
        let defaults = vec![false; multiselected.len()];
        selections = MultiSelect::new()
            .with_prompt(format!("Choisis tes horaires de Cours {DISCLAIMER}"))
            .items(&multiselected[..])
            .defaults(&defaults[..])
            .interact()
            .unwrap();
    }

    // Keep only wanted courses
    for day in &mut timetable.1 .1 {
        day.courses.retain(|course_opt| {
            if let Some(course) = course_opt {
                // Keep if it's a TD/TP
                if course.category.contains(&Category::TD)
                    || course.category.contains(&Category::TP)
                {
                    return true;
                }

                // Keep if only one slot is available
                if *counts.get(&entry_getter(course)).unwrap() == 1 {
                    return true;
                }

                // Keep only chosen courses if multiple was available
                for i in &selections {
                    if get_selection(&(course, day.name.clone())) == multiselected[*i] {
                        return true;
                    }
                }
            }

            false
        });
    }
}

/// Filter the multiples TD/TP
fn tdtp(timetable: &mut Timetable, merge: bool) {
    // If we differentiate TD from TP
    let entry_getter = if merge { get_entry_nocat } else { get_entry };

    // List of TP/TD and Counter of how much they appears
    // to know if multiples slots are available
    let (mut td_or_tp, counts) = get_count(timetable, &[Category::TD, Category::TP], entry_getter);

    // Keep only elements who have multiples TD/TP
    td_or_tp.retain(|course| *counts.get(&entry_getter(course.0)).unwrap() > 1);

    let mut multiselected: Vec<String> = td_or_tp.iter().map(get_selection).collect();
    multiselected.sort();

    let mut selections = vec![];
    if !multiselected.is_empty() {
        let defaults = vec![false; multiselected.len()];
        selections = MultiSelect::new()
            .with_prompt(format!("Choisis tes horaires de TD/TP {DISCLAIMER}"))
            .items(&multiselected[..])
            .defaults(&defaults[..])
            .interact()
            .unwrap();
    }

    // Keep only wanted courses
    for day in &mut timetable.1 .1 {
        day.courses.retain(|course_opt| {
            if let Some(course) = course_opt {
                // Keep if it's a course
                if course.category.contains(&Category::Cours) {
                    return true;
                }

                // Keep if only one slot is available of the TD/TP
                if *counts.get(&entry_getter(course)).unwrap() == 1 {
                    return true;
                }

                // Keep only chosen TD/TP if multiple was available
                for i in &selections {
                    if get_selection(&(course, day.name.clone())) == multiselected[*i] {
                        return true;
                    }
                }
            }

            false
        });
    }
}
