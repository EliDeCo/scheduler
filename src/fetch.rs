use crate::schedule::{get_days, to_military};
use crate::structs::*;
use futures::{StreamExt, stream};
use reqwest::Client;
use std::collections::HashMap;
const CONCURRENCY: usize = 10;

///returns the sectionmap for a given course and semester from the UMD API
pub async fn get_sections(
    course_id: &String,
    semester_id: &String,
) -> Result<SectionMap, Box<dyn std::error::Error>> {
    let client: Client = Client::new();
    let raw: String = client
        .get(format!(
            "https://api.umd.io/v1/courses/{}/sections?semester={}",
            course_id, semester_id
        ))
        .send()
        .await?
        .text()
        .await?;
    //collection data into the input struct
    let input_list: Vec<SectionInput> = serde_json::from_str(&raw)?;
    let mut output_map: SectionMap = HashMap::new();

    //format it into the output
    for section_input in input_list {
        //iterate through each section
        //format the Classtimes struct
        let mut classtimes: Classtimes = HashMap::new();
        for meeting in section_input.meetings {
            //iterate through each meeting group
            // format the start and end time of this meet group
            let start_end: StartEnd = StartEnd {
                building: meeting.building,
                start: to_military(meeting.start_time),
                end: to_military(meeting.end_time),
            };
            //get the days where this meet group occurs
            let days: Vec<u32> = get_days(meeting.days);
            //use the times and days to add an entries to the classtimes property for this section
            for day in days {
                //classtimes.insert(day, start_end.clone());
                if let Some(day_meetings) = classtimes.get_mut(&day) {
                    day_meetings.push(start_end.clone());
                } else {
                    classtimes.insert(day, vec![start_end.clone()]);
                }
            }
        }

        let course_name: String = section_input.course;
        let section_name: String = section_input.number;
        let professor: String = match section_input.instructors.is_empty() {
            true => String::from("Unknown"),
            false => section_input.instructors[0].clone(),
        };
        let seats: [u32; 3] = [
            section_input.seats.parse().unwrap_or_default(),
            section_input.open_seats.parse().unwrap_or_default(),
            section_input.waitlist.parse().unwrap_or_default(),
        ];

        //compile section struct, formatted for output
        let section_formatted: Section = Section {
            professor: ProfData {
                name: professor,
                rating: 0.0,
            },
            classtimes: classtimes,
            course: course_name,
            section: section_name.clone(),
            seats: seats,
        };
        output_map.insert(section_name, section_formatted);
    }

    return Ok(output_map);
}

///Fetches all courses concurrently and returns a CourseMap
pub async fn fetch_all_courses(ideal_courses: &[String], semester: &String) -> CourseMap {
    let results: Vec<(String, Result<SectionMap, Box<dyn std::error::Error>>)> =
        stream::iter(ideal_courses.iter().cloned())
            .map(|course: String| {
                let sem = semester.clone();
                async move {
                    let res = get_sections(&course, &sem).await;
                    //retry up to 3 times if there was an error
                    let mut count: i8 = 0;
                    while res.is_err() && count < 3 {
                        count += 1;
                        println!(
                            "Retrying fetch for course {} (attempt {})",
                            course,
                            count + 1
                        );
                        let res_retry = get_sections(&course, &sem).await;
                        if res_retry.is_ok() {
                            return (course, res_retry);
                        }
                    }
                    //return the final result (either success or failure)
                    (course, res)
                }
            })
            .buffer_unordered(CONCURRENCY)
            .collect()
            .await;

    let mut all_courses: CourseMap = HashMap::new();
    for (course, secs_res) in results {
        match secs_res {
            Ok(secs) => {
                println!("Successfully retrieved course {}", course);
                all_courses.insert(course, secs);
            }
            Err(e) => {
                println!("Error retrieving course {}: {}", course, e);
            }
        }
    }
    all_courses
}
