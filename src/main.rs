use clap::Parser;
use dialoguer::Input;
use regex::Regex;

mod filter;
mod ics;
mod info;
mod timetable;
mod utils;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    /// The class you want to get the timetable, i.e.: M1
    #[clap(value_parser)]
    class: String,

    /// The semester you want (1 or 2), default to current semester
    #[clap(short, long, value_parser, value_name = "SEMESTER NUMBER")]
    semester: Option<i8>,

    /// The year, default to current year
    #[clap(short, long, value_parser, value_name = "YEAR")]
    year: Option<i32>,

    /// Export to iCalendar format (.ics)
    #[clap(short, long, value_name = "FILE NAME")]
    export: Option<String>,

    /// Size of cell of the timetable (irrelevant when exporting)
    #[clap(short, long, value_name = "CELL LENGTH", default_value_t = 35)]
    cl: usize,

    /// Doesn't distinguish TD from TP
    #[clap(short, long)]
    td_are_tp: bool,

    /// First day of your year
    #[clap(short, long)]
    first_day: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let matches = Regex::new(r"(?i)M(?P<level>[1,2])")
        .unwrap()
        .captures(&args.class)
        .unwrap();

    let level = matches
        .name("level")
        .unwrap()
        .as_str()
        .parse::<i8>()
        .unwrap();

    let user_agent = format!("cal7tor/{}", env!("CARGO_PKG_VERSION"));

    println!("Récupération de l'emploi du temps des M{level}...");
    let mut timetable = timetable::timetable(level, args.semester, args.year, &user_agent).await;

    timetable = filter::timetable(timetable, args.td_are_tp);

    let date = match args.first_day {
        None => Input::new()
            .with_prompt("Début des cours de l'année")
            .default(info::get_start_date(level, args.semester, args.year, &user_agent).await)
            .interact_text()
            .unwrap(),
        Some(day) => day,
    };

    println!("Récupération des informations par rapport à l'année...");
    let info = info::info(args.semester, args.year, &date);

    if args.export.is_some() {
        // Export the calendar
        let mut filename = args.export.unwrap();

        let builded_timetable = timetable::build(&timetable, &info);
        ics::export(builded_timetable, &mut filename);

        println!("Fichier .ICS construit et exporté => {filename}");
    } else {
        println!("\x1b[93mNOTICE: IT WON'T WORK!!!\x1b[0m");
        // Show the calendar
        println!("Affichage...");
        timetable::display(&timetable, args.cl);
        println!("Vous devrez peut-être mettre votre terminal en plein écran si ce n'est pas déjà le cas.");
    }
}
