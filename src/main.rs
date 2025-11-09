pub mod structs;
pub mod schedule;
pub mod fetch;
pub mod web;
use structs::*;
use schedule::{rating, format_alternates, is_conflict};
use fetch::fetch_all_courses;
use axum::{Router, response::Html, routing::get};
use minijinja::{Environment, context, path_loader};
use std::{
    collections::HashMap,
    fs::File,
    path::Path,
    sync::Arc,
    io::Write
};

//TODO: Starting with one random section does not work
//TODO: Add custom courses
//TODO: DESIGN UI (Have it be on a website)
//TODO: Update seat values for each given course
//TODO: Add in professor ratings
// ship after this one, then add features
//TODO: Add lunch break option
//TODO: Sort by walking distance
//TODO: Professor rating multiplier
//TODO: Add Gened suggestor
//TODO: AI suggestions

//127.0.0.1:7878/display

//walk speed in meters per second
const WALK_SPEED: f32 = 1.42;
//earlist and latest time to go to class
const EARLIST: u32 = 900;
const LATEST: u32 = 1700;


#[tokio::main]
async fn main() {
    //INPUTS============================================================================================
    //courses that must be in every schedule
    let required: Vec<String> = vec![
        "PHYS260".to_string(),
        "ENES200".to_string(),
        //"UMRO".to_string(),
    ];

    //courses that must fit in every schedule but are eligible to have alternates
    let preferred: Vec<String> = vec![
        "ENME272".to_string(),
        "ENME201".to_string(),
        "ENES102".to_string(),
    ];

    //alternates can replace preferred courses as long as they don't overlap with required ones
    let alternates: Vec<String> = vec![
        "FREN103".to_string(),
        "COMM107".to_string(),
        "SPAN204".to_string(),
    ];

    let semester: String = String::from("202601");
    //==================================================================================================


    let ideal_courses: Vec<String> = [required.clone(), preferred.clone()].concat();

    //get building information
    let path: &Path = Path::new("cache/buildings.json");
    let file: File = File::open(path).expect("Failed to open buildings file");
    let buildings: HashMap<String, BuildingData> =
        serde_json::from_reader(file).expect("Json parsing error");

    //get courses
    let all_courses: CourseMap = fetch_all_courses(&ideal_courses, semester).await;

    let desired_courses: CourseMap = all_courses
        .iter()
        .filter(|(course, _)| ideal_courses.contains(course))
        .map(|(course, sections)| (course.clone(), sections.clone()))
        .collect();

    let alternate_courses: CourseMap = all_courses
        .iter()
        .filter(|(course, _)| alternates.contains(course))
        .map(|(course, sections)| (course.clone(), sections.clone()))
        .collect();

    let mut potential_schedules: Vec<Schedule> = Vec::new();
    let mut count: u32 = 0;

    /*
    //for testing
    let my_schedule: Schedule = vec![
        desired_courses["FREN103"]["0301"].clone(),
        desired_courses["ENES102"]["0301"].clone(),
        desired_courses["PHYS260"]["0201"].clone(),
        desired_courses["ENME272"]["0301"].clone(),
        desired_courses["ENES200"]["0101"].clone(),
    ];
    */

    //generate ideal schedules
    'mainloop: for (_, sections) in desired_courses {
        count += 1;
        if count == 1 {
            //initialize the potential schedules with the first course, then move on to the next course
            potential_schedules.push(vec![sections.values().next().unwrap().clone()]);
            continue 'mainloop;
        }

        let mut new_potential_schedules: Vec<Schedule> = Vec::new();
        for (_, new_section) in sections {
            'schedule_loop: for schedule in potential_schedules.clone() {
                for section in schedule.clone() {
                    if is_conflict(
                        &section,
                        &new_section,
                        &buildings,
                        WALK_SPEED,
                        EARLIST,
                        LATEST,
                    ) {
                        continue 'schedule_loop;
                    }
                }
                //if we reach here, every section in the currently selected schedule is compatible with the new section
                //this means that this schedule is valid, as it can hold 1 of every course we have iterated through at this point in time
                let mut new_schedule: Vec<Section> = schedule.clone();
                new_schedule.push(new_section.clone());
                //sort courses alphabetically within their schedules
                new_schedule.sort_by(|a, b| a.course.cmp(&b.course));
                new_potential_schedules.push(new_schedule);
            }
        }
        potential_schedules = new_potential_schedules;
    }

    let mut to_save: String = String::from("Viable Schedules\n");
    for (i, schedule) in potential_schedules.iter().enumerate() {
        to_save.push_str(&format!("\nSchedule {}:\n", i + 1));
        for section in schedule {
            to_save.push_str(&format!(
                "{}-{}, {}\n",
                section.course, section.section, section.professor.name
            ));
        }
    }

    // for testing
    let mut file: File = File::create("viable.txt").expect("Error with output file creation");
    file.write_all(to_save.as_bytes())
        .expect("Error with output file writing");

    /*
    //for testing
    for (i1, section1) in my_schedule.iter().enumerate() {
        for (i2, section2) in my_schedule.iter().enumerate() {
            if i1 != i2 {
                if is_conflict(section1, section2, &buildings, WALK_SPEED, earliest, latest) {
                    //println!("Conflict between {}-{} and {}-{}", section1.course, section1.section, section2.course, section2.section);
                } else {
                    //println!("all good");
                }
            }
        }
    }
    */

    //A list of schedules, with each schedule being a list of sections, each of which has an associated list of alternates
    let mut schedules_with_alternates: Vec<Vec<(Section, Vec<Section>)>> = Vec::new();
    for schedule in potential_schedules {
        //generate the schedule with all possible alternates
        let mut single_with_alts: Vec<(Section, Vec<Section>)> = schedule
            .iter()
            .map(|s| {
                (
                    s.clone(),
                    s.find_alt(
                        schedule.clone(),
                        &buildings,
                        WALK_SPEED,
                        EARLIST,
                        LATEST,
                        &alternate_courses,
                        &required,
                    ),
                )
            })
            .collect();

        /*
        //Below, excessive alternates are removed
        //if an alternate section belongs to multiple courses, the course with the least alternate option gets to keep it
        for (_, alt_course) in &alternate_courses { //for every alternate course
            for (_, alt_section) in alt_course { //for every section in that alternate course
                //sort the schedule by how many alternatives each course has, least to most
                single_with_alts.sort_by(|a,b| a.1.len().cmp(&b.1.len()));
                let mut keep: bool = true;
                for (_, alts) in single_with_alts.iter_mut() { //for each course in the sorted schedule
                    if alts.contains(alt_section) { //if this course has the alternate in question
                        if keep { //this is the first (and shortest) list with this alternate, so it keeps it
                            keep = false;
                        } else { //this course must drop this alternative, as a different (shorter) course got to keep it
                            alts.retain(|s| s != alt_section);
                        }
                    }
                }
            }
        }
        */
        //sort alphabetically by course
        single_with_alts.sort_by(|a, b| a.0.course.cmp(&b.0.course));

        schedules_with_alternates.push(single_with_alts);
    }

    //format for display

    //sort by rating, highest to lowest
    schedules_with_alternates.sort_by(|a, b| {
        rating(b, &alternates)
            .partial_cmp(&rating(a, &alternates))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut all_schedules: Vec<Vec<DisplaySection>> = Vec::new();

    for schedule in schedules_with_alternates {
        all_schedules.push(
            schedule
                .iter()
                .map(|(s, a)| DisplaySection {
                    professor: s.professor.clone(),
                    classtimes: s.humanize_times(),
                    course: s.course.clone(),
                    section: s.section.clone(),
                    seats: s.seats.clone(),
                    alternates: {
                        //consolidate excess alternates and format for display
                        if a.is_empty() {
                            String::from("N/A")
                        } else {
                            format_alternates(a, 4)
                        }
                    },
                })
                .collect(),
        );
    }

    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    let env = Arc::new(env);

    let app = Router::new().route(
        "/display",
        get({
            let env = env.clone();
            move || {
                let env = env.clone();
                async move {
                    let tmpl = env.get_template("display.html").unwrap();
                    let rendered = tmpl.render(context! { all_schedules }).unwrap();
                    Html(rendered)
                }
            }
        }),
    );

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind("127.0.0.1:7878")
        .await
        .unwrap();
    println!("Launching webpage");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
