use std::sync::Arc;

use chrono::TimeZone;
use ics::{
    parameters::{Language, PartStat, Role, TzIDParam, CN},
    properties::{
        Attendee, Categories, Class, Description, DtEnd, DtStart, Location, Summary, Transp,
    },
    Event, ICalendar, Standard,
};

pub fn export(
    courses: Vec<crate::timetable::models::Course>,
    filename: &mut String,
    with_tz: bool,
) {
    let mut calendar = ICalendar::new("2.0", "cal7tor");

    // Add Europe/Paris timezone
    let timezone_name = "Europe/Paris";
    if with_tz {
        calendar.add_timezone(ics::TimeZone::standard(
            timezone_name,
            Standard::new(
                // Add a Z because it's UTC
                dt_ical(chrono::Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()) + "Z",
                "+0100",
                "+0200",
            ),
        ));
    }

    // Create events which contains the information regarding the course
    for course in courses {
        let mut event = Event::new(
            uuid::Uuid::new_v4().to_string(),
            // Add a Z because it's UTC
            dt_ical(chrono::Utc::now()) + "Z",
        );

        // Public event
        event.push(Class::public());

        // Consume actual time
        event.push(Transp::opaque());

        // Professor's name
        if course.professor.is_some() {
            let name = course.professor.unwrap();
            let mut contact = Attendee::new("mailto:place@holder.com");
            contact.add(CN::new(name));
            contact.add(PartStat::ACCEPTED);
            contact.add(Role::CHAIR);
            event.push(contact);
        }

        // Start time of the course
        let mut date_start = DtStart::new(dt_ical(course.dtstart.unwrap()));
        if with_tz {
            date_start.add(TzIDParam::new(timezone_name));
        }
        event.push(date_start);

        // End time of the course
        let mut date_end = DtEnd::new(dt_ical(course.dtend.unwrap()));
        if with_tz {
            date_end.add(TzIDParam::new(timezone_name));
        }
        event.push(date_end);

        // Room location
        event.push(Location::new(course.room));

        let categories = course
            .category
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Arc<[String]>>()
            .join("/");

        // Course's name
        let mut course_name = Summary::new(format!("{} - {}", categories, course.name));
        course_name.add(Language::new("fr"));
        event.push(course_name);

        // Course's category
        event.push(Categories::new(categories));

        // Course extra data
        if course.data.is_some() {
            event.push(Description::new(course.data.unwrap()));
        }

        // Add the course to the calendar
        calendar.add_event(event);
    }

    // Add the extension if needed
    if !std::path::Path::new(filename)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("ics"))
    {
        *filename = format!("{filename}.ics");
    };

    calendar.save_file(filename).unwrap();
}

/// Transform the datetime from chrono to the ICS format
/// See <https://github.com/hummingly/ics/issues/17#issue-985662287>
fn dt_ical(dt: chrono::DateTime<chrono::Utc>) -> String {
    format!("{}", dt.format("%Y%m%dT%H%M%S"))
}
