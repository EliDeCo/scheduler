use crate::schedule::{is_conflict, un_military_time};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
pub type CourseMap = HashMap<String, SectionMap>;
pub type SectionMap = HashMap<String, Section>;
pub type Classtimes = HashMap<u32, Vec<StartEnd>>;
pub type ClasstimesForHumans = Vec<String>;
pub type ScheduleWithAlternates = Vec<(Section, Vec<Section>)>; // a schedule where each section has a list of alternates
pub type BuildingMap = HashMap<String, BuildingData>;
pub type DisplaySchedule = Vec<DisplaySection>;

pub type Schedule = Vec<Section>;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Section {
    pub professor: ProfData,
    pub classtimes: Classtimes,
    pub course: String,
    pub section: String,
    pub seats: [u32; 3], //Total, open, waitlisted
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StartEnd {
    pub building: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Deserialize)]
pub struct BuildingData {
    //pub name: String,
    //pub id: String,
    pub long: f32,
    pub lat: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfData {
    pub name: String,
    pub rating: f32,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct DisplaySection {
    pub professor: ProfData,
    pub classtimes: ClasstimesForHumans,
    pub course: String,
    pub section: String,
    pub seats: [u32; 3], //Total, open, waitlisted
    pub alternates: String,
}

//This is the type that the API returns
#[derive(Debug, Deserialize)]
pub struct SectionInput {
    pub course: String,
    //section_id: String,
    //semester: String,
    pub number: String,
    pub seats: String,
    pub meetings: Vec<MeetTime>,
    pub open_seats: String,
    pub waitlist: String,
    pub instructors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MeetTime {
    pub days: String,
    //room: String,
    pub building: String,
    //classtype: String,
    pub start_time: String,
    pub end_time: String,
}

impl PartialEq for Section {
    fn eq(&self, _other: &Self) -> bool {
        self.course == _other.course && self.section == _other.section
    }
}

impl Section {
    ///Finds an alternate sections that can replace this section in the given schedule
    pub fn find_alt(
        &self,
        mut schedule: Vec<Section>,
        buildings: &HashMap<String, BuildingData>,
        walk_speed: f32,
        earliest: u32,
        latest: u32,
        alternates: &CourseMap,
        required: &Vec<String>,
    ) -> Vec<Section> {
        //if this course is required, no alternates will be given, otherwise continue with the logic
        if required.contains(&self.course) {
            return Vec::new();
        }

        //remove the course in question
        schedule.retain(|s| s != self);

        //test every alternate and keep track of the ones that fit properly
        let mut alts: Vec<Section> = Vec::new();
        for (_, alt_section_map) in alternates {
            //for each alternate course
            'section_loop: for (_, alt_section) in alt_section_map {
                //for each section in that alternate course
                for current_section in &schedule {
                    //see if the alternate section conflicts with any other section in the current schedule
                    if is_conflict(
                        current_section,
                        alt_section,
                        buildings,
                        walk_speed,
                        earliest,
                        latest,
                    ) {
                        continue 'section_loop; //if this section conflicts with anything in the schedule, move on to the next section
                    }
                }
                //if we reach here, that means this alternate is compatible with the whole schedule
                alts.push(alt_section.clone());
            }
        }

        return alts;
    }
    ///Takes class times stored with numbers for computers to stored by days for humans
    pub fn humanize_times(&self) -> ClasstimesForHumans {
        let mut classtimes_human: HashMap<String, Vec<String>> = HashMap::new();
        let key: HashMap<String, usize> = HashMap::from([
            ("M".to_string(), 0),
            ("Tu".to_string(), 1),
            ("W".to_string(), 2),
            ("Th".to_string(), 3),
            ("F".to_string(), 4),
        ]);
        for (day_num, times) in &self.classtimes {
            let day_str: &'static str = match day_num {
                1 => "M",
                2 => "Tu",
                3 => "W",
                4 => "Th",
                5 => "F",
                _ => "Unknown",
            };

            for time in times {
                let time_str = format!(
                    "{}-{} in {}",
                    un_military_time(time.start),
                    un_military_time(time.end),
                    time.building
                );
                if let Some(meeting) = classtimes_human.get_mut(&time_str) {
                    meeting.push(day_str.to_string());
                } else {
                    classtimes_human.insert(time_str, vec![day_str.to_string()]);
                }
            }
        }

        //sort days in chronological order
        for (_, days) in classtimes_human.iter_mut() {
            let mut days_sorted: Vec<String> = vec![
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ];
            for day in days.iter() {
                days_sorted[key[day.as_str()]] = day.to_string();
            }
            days_sorted.retain(|d| !d.is_empty());
            *days = days_sorted;
        }

        return classtimes_human
            .into_iter()
            .map(|(time, days)| days.join("") + " " + &time)
            .collect();
    }
}
