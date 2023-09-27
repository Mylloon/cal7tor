use clap::Parser;
use regex::Regex;

mod ics;
mod info;
mod timetable;
mod utils;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    /// The class you want to get the timetable, i.e.: M1-LP
    #[clap(value_parser)]
    class: String,

    /// The semester you want (1 or 2)
    #[clap(short, long, value_parser, value_name = "SEMESTER NUMBER")]
    semester: Option<i8>,

    /// The year, default to the current year
    #[clap(short, long, value_parser, value_name = "YEAR")]
    year: Option<i32>,

    /// Export to iCalendar format (.ics)
    #[clap(short, long, value_name = "FILE NAME")]
    export: Option<String>,

    /// Size of cell of the timetable (irrelevant when exporting the timetable)
    #[clap(short, long, value_name = "CELL LENGTH", default_value_t = 35)]
    cl: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let matches =
        Regex::new(r"(?i)M(?P<level>[1,2])[-–•·]?(?P<pathway>(LP|IMPAIRS|DATA|GENIAL|MPRI))?")
            .unwrap()
            .captures(&args.class)
            .unwrap();

    let level = matches
        .name("level")
        .unwrap()
        .as_str()
        .parse::<i8>()
        .unwrap();
    let pathway = matches.name("pathway").unwrap().as_str();

    let user_agent = format!("cal7tor/{}", env!("CARGO_PKG_VERSION"));

    println!(
        "Récupération de l'emploi du temps des M{}-{}...",
        level,
        pathway.to_uppercase()
    );
    let timetable = timetable::timetable(level, args.semester, args.year, &user_agent).await;

    println!("Récupération des informations par rapport à l'année...");
    let info = info::info(level, args.semester, args.year, &user_agent).await;

    if args.export.is_some() {
        // Export the calendar
        let mut filename = args.export.unwrap();

        let builded_timetable = timetable::build(timetable, info);
        ics::export(builded_timetable, &mut filename);

        println!("Fichier .ICS construit et exporté => {}", filename);
    } else {
        // Show the calendar
        println!("Affichage...");
        timetable::display(timetable, args.cl);
        println!("Vous devrez peut-être mettre votre terminal en plein écran si ce n'est pas déjà le cas.");
    }
}
