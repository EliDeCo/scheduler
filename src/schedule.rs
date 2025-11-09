use crate::structs::*;
use haversine_rs::{distance, point::Point, units::Unit};
use std::collections::HashMap;

//walk speed in meters per second
const WALK_SPEED: f32 = 1.42;
//earlist and latest time to go to class
const EARLIST: u32 = 900;
const LATEST: u32 = 1700;

///Turns computer formatted time into human formatted time
pub fn un_military_time(time: u32) -> String {
    let mut to_return: String;
    if time >= 1300 {
        to_return = (time - 1200).to_string() + "pm";
    } else if time == 1200 {
        to_return = time.to_string() + "pm";
    } else {
        to_return = time.to_string() + "am";
    }

    to_return.insert(to_return.len() - 4, ':');
    to_return
}
/// Convert HHMM -> total minutes since midnight and compare
pub fn time_between(first: u32, second: u32) -> u32 {
    let to_minutes = |time: u32| -> u32 {
        let hours = time / 100;
        let minutes = time % 100;
        hours * 60 + minutes
    };

    return to_minutes(second) - to_minutes(first);
}

/// takes two sections and determines if they have overlapping time slots, unwalkable, or too early or late
pub fn is_conflict(
    section1: &Section,
    section2: &Section,
    buildings: &HashMap<String, BuildingData>,
    walk_speed: f32,
    earliest: u32,
    latest: u32,
) -> bool {
    //TO DEBUG:
    let tester: bool;
    if (0 == 1)
        && section1.course == "FREN103"
        && section1.section == "0301"
        && section2.course == "PHYS260"
        && section2.section == "0201"
    {
        tester = true;
    } else {
        tester = false;
    }

    for day in 1..6 {
        //if the day is present in both sections
        if let (Some(times_from_1), Some(times_from_2)) =
            (section1.classtimes.get(&day), section2.classtimes.get(&day))
        {
            //compare every meeting on every day from both courses with each other
            for times1 in times_from_1 {
                for times2 in times_from_2 {
                    //if the class starts or ends to early, this section is deemed a conflict
                    if times1.start < earliest
                        || times2.start < earliest
                        || times1.end > latest
                        || times2.end > latest
                    {
                        if tester {
                            println!(
                                "On day {}, {}-{} OR {}-{} ends too late/starts too early",
                                day,
                                section1.course,
                                section1.section,
                                section2.course,
                                section2.section
                            );
                        }
                        return true;
                    }

                    //if any start or end times are shared, they overlap
                    if times1.start == times2.start
                        || times1.start == times2.end
                        || times1.end == times2.start
                        || times1.end == times2.end
                    {
                        if tester {
                            println!(
                                "On day {}, {}-{} conflicts with {}-{}: Start/End Shared",
                                day,
                                section1.course,
                                section1.section,
                                section2.course,
                                section2.section
                            );
                        }
                        return true;
                    }

                    //order them by which one starts first
                    let mut chronological: [&StartEnd; 2] = [times1, times2];
                    chronological.sort_by(|a, b| a.start.cmp(&b.start));

                    let first: &StartEnd = chronological[0];
                    let second: &StartEnd = chronological[1];

                    //if the first one ends after the second one starts, they overlap
                    if first.end > second.start {
                        if tester {
                            println!(
                                "On day {}, {}-{} conflicts with {}-{}: Overlap Trouble",
                                day,
                                section1.course,
                                section1.section,
                                section2.course,
                                section2.section
                            );
                        }
                        return true;
                    } else {
                        //test to see if there is enough time to walk
                        let time_between: u32 = time_between(first.end, second.start) * 60; //time between classes in seconds

                        let pos1: Point = Point::new(
                            buildings[&first.building].lat as f64,
                            buildings[&first.building].long as f64,
                        );
                        let pos2: Point = Point::new(
                            buildings[&second.building].lat as f64,
                            buildings[&first.building].long as f64,
                        );
                        let pos3: Point = Point::new(
                            buildings[&second.building].lat as f64,
                            buildings[&second.building].long as f64,
                        );

                        //computes the maximum distance: we cannot take a straghit line, and must go straight East or West, then straight north or south
                        //this simulates real walking where we often have to follow horizontal and vertical roads and paths
                        let max_distance: f32 = distance(pos1, pos2, Unit::Meters) as f32
                            + distance(pos2, pos3, Unit::Meters) as f32;
                        //println!("Distance between {} and {} = {}", first.building, second.building, max_distance);
                        /*
                        if tester && time_between == 600 {
                            println!("-----------------------------------");
                            println!("Distance between {} and {} = {}", first.building, second.building, max_distance);
                            let walk_time = max_distance / walk_speed;
                            println!("Walk time in minutes: {}", walk_time/60.);

                            //println!("How early in minutes: {}", (time_between as f32 /60.) -(walk_time/60.))
                            println!("{}-{}, {}-{}", first.start, first.end, second.start, second.end);
                            println!("Time between: {}", time_between/60)
                        }
                        */

                        if time_between as f32 - (max_distance / walk_speed) < 300. {
                            //if we can't get there 5 minutes early, deem this section as a conflict
                            if tester {
                                println!(
                                    "On day {}, {}-{} conflicts with {}-{}: Cant get there in time",
                                    day,
                                    section1.course,
                                    section1.section,
                                    section2.course,
                                    section2.section
                                );
                            }
                            //println!("Cant walk");
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

///gives a rating of the inputted schedule for ordering
pub fn rating(schedule: &ScheduleWithAlternates, all_alternates: &Vec<String>) -> f32 {
    //Sum of all professor ratings
    let prof_rating: f32 = schedule.iter().map(|(s, _)| s.professor.rating).sum();

    //list of the average ratings for each alternate
    let alt_ratings: Vec<f32> = schedule
        .iter()
        .map(|(_, a)| {
            if a.is_empty() {
                0.0
            } else {
                let sum: f32 = a.iter().map(|s| s.professor.rating).sum();
                sum / a.len() as f32
            }
        })
        .collect();

    //average alternate rating
    let av_alt_rating: f32 = alt_ratings.iter().sum::<f32>() / alt_ratings.len() as f32;

    //rewards a schedule for giving freedom in which alternate courses are availible and when they can be taken
    let mut alternate_diversity_rating: f32 = 0.;
    for (_, alts) in schedule {
        //for each course in the schedule
        let course_alts: Vec<String> = alts.iter().map(|s| s.course.clone()).collect();
        let mut counts: HashMap<String, usize> = HashMap::new(); //Amount of times each alternate course shows up
        for alt in course_alts {
            *counts.entry(alt).or_insert(0) += 1;
        }
        //get reward based on how many alternate courses (not sections) are availible for this course
        alternate_diversity_rating += counts.len() as f32;
        //get a reward based on the median number of sections per alternate course (rewards diverse options without overvaluing outliers)
        for given_alternate in all_alternates.clone() {
            //insert  zeroes for sections not included
            counts.entry(given_alternate).or_insert(0);
        }
        let section_nums: Vec<f32> = counts.values().copied().map(|v| v as f32).collect();
        alternate_diversity_rating += median(&section_nums);
    }

    return prof_rating + av_alt_rating + alternate_diversity_rating;
}

///Formats alternates to be nice on the eyes
pub fn format_alternates(sections: &Vec<Section>, threshold: usize) -> String {
    //count occurrences
    let mut counts: HashMap<&String, usize> = HashMap::new();
    for s in sections {
        *counts.entry(&s.course).or_default() += 1;
    }

    //reconstruct through occurences
    let mut seen: Vec<String> = Vec::new();
    let mut output: Vec<String> = Vec::new();
    for s in sections {
        let course = s.course.clone();
        let count = counts.get(&course).copied().unwrap_or(0);
        if count > threshold {
            //compact form showing abundance of section options
            if !seen.contains(&course) {
                output.push(format!("{}: XXXX, ", course));
                seen.push(course);
            }
        } else {
            //normal format
            output.push(format!("{}: {}, ", course, s.section));
        }
    }

    output.sort();

    return output.join("");
}

///compute median of a collection of floats
fn median(numbers: &Vec<f32>) -> f32 {
    let mut numbers = numbers.clone();
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = numbers.len();

    if len == 0 {
        return f32::NAN; // or panic!("empty list") if that fits your case
    }

    if len % 2 == 1 {
        numbers[len / 2]
    } else {
        let mid = len / 2;
        (numbers[mid - 1] + numbers[mid]) / 2.0
    }
}

///converts the given convential time in string form into military time in integer form
pub fn to_military(mut time: String) -> u32 {
    let time2: String = time.clone();
    time.retain(|c: char| c.is_numeric());
    let nums: u32 = time.parse().unwrap_or_default();
    if time2.contains("p") {
        if nums >= 1200 && nums <= 1259 {
            return nums;
        } else {
            return nums + 1200;
        }
    } else if time2.contains("a") {
        return nums;
    } else {
        //happens when the time is unknown
        //atleast I think so?
        return 0;
    }
}

///converts the given days in string form into a vector of numbers representing those days
pub fn get_days(input: String) -> Vec<u32> {
    let mut output: Vec<u32> = Vec::new();
    let mut buffer: String = String::new();
    for char in input.chars() {
        if char == 'M' {
            output.push(1);
        } else if char == 'W' {
            output.push(3);
        } else if char == 'F' {
            output.push(5);
        } else {
            //if it is one of the two letter combinations, use the buffer to aid in recognition
            buffer.push(char);
            if buffer == String::from("Tu") {
                output.push(2);
                buffer.clear();
            } else if buffer == String::from("Th") {
                output.push(4);
                buffer.clear();
            }
        }
    }

    output
}

///Generates all potential schedules from the desired courses
pub fn get_potential_schedules(
    desired_courses: CourseMap,
    buildings: &BuildingMap,
) -> Vec<Schedule> {
    let mut potential_schedules: Vec<Schedule> = Vec::new();
    let mut count: u32 = 0;

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

    potential_schedules
}

///Computes possible alternates for all the given potential schedules
pub fn schedules_with_alternatives(
    potential_schedules: Vec<Schedule>,
    buildings: &BuildingMap,
    alternates: &CourseMap,
    required: &Vec<String>,
) -> Vec<ScheduleWithAlternates> {
    let mut schedules_with_alternates: Vec<ScheduleWithAlternates> = Vec::new();
    for schedule in potential_schedules {
        //generate the schedule with all possible alternates
        let mut single_with_alts: Vec<(Section, Vec<Section>)> = schedule
            .iter()
            .map(|s| {
                (
                    s.clone(),
                    s.find_alt(
                        schedule.clone(),
                        buildings,
                        WALK_SPEED,
                        EARLIST,
                        LATEST,
                        alternates,
                        required,
                    ),
                )
            })
            .collect();

        //sort alphabetically by course
        single_with_alts.sort_by(|a, b| a.0.course.cmp(&b.0.course));

        schedules_with_alternates.push(single_with_alts);
    }

    //sort by rating, highest to lowest
    let alternates: Vec<String> = alternates.keys().cloned().collect();
    schedules_with_alternates.sort_by(|a, b| {
        rating(b, &alternates)
            .partial_cmp(&rating(a, &alternates))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    schedules_with_alternates
}

///Formats schedules with alternates for display
pub fn schedules_for_display(
    schedules_with_alternates: Vec<ScheduleWithAlternates>,
) -> Vec<DisplaySchedule> {
    let mut all_schedules: Vec<DisplaySchedule> = Vec::new();
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
    all_schedules
}
