use chrono::TimeZone;
use ics::{
    parameters,
    properties::{Categories, Class, Description, DtEnd, DtStart, Location, Summary, Transp},
    Event, ICalendar, Standard,
};

pub fn export(courses: Vec<crate::timetable::models::Course>, filename: &mut String) {
    let mut calendar = ICalendar::new("2.0", "cal8tor");

    // Add Europe/Paris timezone
    let timezone_name = "Europe/Paris";
    calendar.add_timezone(ics::TimeZone::standard(
        timezone_name,
        Standard::new(
            // Add a Z because it's UTC
            dt_ical(chrono::Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()) + "Z",
            "+0100",
            "+0200",
        ),
    ));

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
            event.push(Description::new(course.professor.unwrap()));
        }

        // Start time of the course
        let mut date_start = DtStart::new(dt_ical(course.dtstart.unwrap()));
        date_start.append(parameters!("TZID" => timezone_name));
        event.push(date_start);

        // End time of the course
        let mut date_end = DtEnd::new(dt_ical(course.dtend.unwrap()));
        date_end.append(parameters!("TZID" => timezone_name));
        event.push(date_end);

        // Room location
        event.push(Location::new(course.room));

        // Course's name
        let mut course_name = Summary::new(format!("{} - {}", course.category, course.name));
        course_name.append(parameters!("LANGUAGE" => "fr"));
        event.push(course_name);

        // Course's category
        event.push(Categories::new(course.category.to_string()));

        // Add the course to the calendar
        calendar.add_event(event);
    }

    // Add the extension if needed
    if !filename.ends_with(".ics") {
        *filename = format!("{}.ics", filename)
    };

    calendar.save_file(filename).unwrap();
}

/// Transform the datetime from chrono to the ICS format
/// See <https://github.com/hummingly/ics/issues/17#issue-985662287>
fn dt_ical(dt: chrono::DateTime<chrono::Utc>) -> String {
    format!("{}", dt.format("%Y%m%dT%H%M%S"))
}
