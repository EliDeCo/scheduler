pub mod fetch;
pub mod schedule;
pub mod structs;
pub mod web;
use fetch::fetch_all_courses;
use schedule::{get_potential_schedules, schedules_for_display, schedules_with_alternatives};
use std::{collections::HashMap, fs::File, path::Path};
use structs::*;
use web::launch_webpage;

//FIX NOW
//TODO: DESIGN UI (Have it be on a website)


//NEXT UP
//TODO: Add blockout times (like work or lunch)


//EVENTUALLY FEATURES
//TODO: Sign in and save favorite schedules
//TODO: Share courses via link
//TODO: Sort by walking distance
//TODO: Professor rating multiplier
//TODO: Add Gened suggestor
//TODO: Let user add custom courses (and have them be saved)
//TODO: AI suggestions
//TODO: Map visualization of schedule



//127.0.0.1:7878/display

#[tokio::main]
async fn main() {
    //INPUTS============================================================================================
    //courses that must be in every schedule
    let desired: Vec<String> = vec![
        "PHYS260".to_string(),
        "ENES200".to_string(),
        "ENME272".to_string(),
        "ENME201".to_string(),
        "ENES102".to_string(),
        //"UMRO".to_string(),
    ];

    //alternates can replace preferred courses as long as they don't overlap with required ones
    let alternates: Vec<String> = vec![
        "FREN103".to_string(),
        "COMM107".to_string(),
        "SPAN204".to_string(),
        "CHBE473".to_string(),
    ];

    let semester: String = String::from("202601");
    //==================================================================================================

    let every_course: Vec<String> =
        [desired.clone(), alternates.clone()].concat();
    let every_course: CourseMap = fetch_all_courses(&every_course, &semester).await;

    let desired_courses: CourseMap = every_course
        .iter()
        .filter(|(k, _)| desired.contains(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let alternate_courses: CourseMap = every_course
        .iter()
        .filter(|(k, _)| alternates.contains(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    //get building information
    let path: &Path = Path::new("cache/buildings.json");
    let file: File = File::open(path).expect("Failed to open buildings file");
    let buildings: HashMap<String, BuildingData> =
        serde_json::from_reader(file).expect("Json parsing error");

    //generate all potential schedules
    let potential_schedules: Vec<Schedule> = get_potential_schedules(desired_courses, &buildings);

    //generate alternates for those schedules
    let schedules_with_alternates: Vec<ScheduleWithAlternates> = schedules_with_alternatives(
        potential_schedules,
        &buildings,
        &alternate_courses,
    );

    //format for display
    let all_schedules: Vec<DisplaySchedule> = schedules_for_display(schedules_with_alternates);

    //Launch webpage to show results
    launch_webpage(all_schedules).await;
}
