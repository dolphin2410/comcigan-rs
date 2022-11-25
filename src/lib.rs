use std::{collections::HashMap};

use bytes::BytesMut;
use class::{Grade, Class, Day, SchoolData};
use client::ComciganClient;
use fancy_regex::Regex;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use anyhow::Result;

pub mod class;
pub mod client;

#[derive(Serialize, Deserialize)]
/// The raw json implementation of the school list
pub struct SchoolList {
    pub(crate) 학교검색: Vec<School>
}

#[derive(Serialize, Deserialize)]
pub struct School(u32, String, String, u32);

#[derive(Serialize, Deserialize)]
/// The raw json implementation of the school data
pub struct RawSchoolData {
    pub timetable: Vec<Vec<Vec<Vec<u32>>>>,
    pub subjects: Vec<String>,
    pub teachers: Vec<String>
}

/// The keys for parsing the `RawSchoolData` struct from obfuscated JSON.
pub struct RawSchoolDataKey {
    pub timetable: String,
    pub subjects: String,
    pub teachers: String,
    pub encode_header: String,
    pub url_piece: String
}

/// Creates timetable data for all classes of all grades
pub async fn view(client: &dyn ComciganClient, school: &School, keys: &RawSchoolDataKey) -> Result<SchoolData> {
    let raw_id = format!("{}{}_0_1", keys.encode_header, school.3);
    let encoded = base64::encode(raw_id);

    let target = keys.url_piece.split("?").nth(0).unwrap();
    let request = format!("http://comci.kr:4082/{}?{}", &target, &encoded).parse()?;

    let mut buffer = BytesMut::with_capacity(1024);
    client.fetch_bytes(request, &mut buffer).await?;

    let (school_list, _, _) = encoding_rs::UTF_8.decode(&buffer[..]);
    let json = validate_json(&school_list);

    // Can't do this in structs because the keys aren't static
    let raw_data = serde_json::from_str::<HashMap<&str, Value>>(json.as_str()).unwrap();
    let teachers = serde_json::value::from_value::<Vec<String>>(raw_data.get(keys.teachers.as_str()).unwrap().to_owned()).unwrap(); // Get teachers list
    let subjects = serde_json::value::from_value::<Vec<String>>(raw_data.get(keys.subjects.as_str()).unwrap().to_owned()).unwrap(); // Get subjects list
    let timetable = serde_json::value::from_value::<Vec<Vec<Vec<Vec<u32>>>>>(raw_data.get(keys.timetable.as_str()).unwrap().to_owned()).unwrap();   // Get timetable list
    
    let data = RawSchoolData {
        teachers,
        subjects,
        timetable
    };

    buffer.clear();

    let mut school_data = SchoolData { name: school.1.clone(), grades: vec![] };  // todo fix school name
    for grade_index in 1..data.timetable.len() {
        let mut grade = Grade::new(grade_index as u8);

        for class_index in 1..data.timetable[grade_index].len() {
            let mut class = Class::new(class_index as u8);

            for days_index in 1..data.timetable[grade_index][class_index].len() {
                let mut day = Day::new(days_index as u8);

                for index in 1..data.timetable[grade_index][class_index][days_index].len() {
                    let subj_data = data.timetable[grade_index][class_index][days_index][index];
                    let th = (subj_data as f32 / 100.0).floor() as u32;
                    let code = subj_data - (th * 100);
                    let mut subject = data.subjects[code as usize].clone();
                    let mut teacher = data.teachers[(th % 100) as usize].clone();
                    if subject == "19" {
                        subject.clear();
                        teacher.clear();
                    }
                    day.periods.push(class::Period { subject, teacher, period_num: index as u8 });
                }
                class.days.push(day)
            }
            grade.classes.push(class);
        }
        school_data.grades.push(grade);
    }

    Ok(school_data)
}

/// Search school from the given string piece
pub async fn search_school(client: &dyn ComciganClient, school: &str, keys: &RawSchoolDataKey) -> Result<Vec<School>> {
    let (result, _, _) = encoding_rs::EUC_KR.encode(school);
    let query: String = result.iter().map(|byte| format!("%{:X}", byte)).collect();

    // Read from the URL
    let request = format!("http://comci.kr:4082/{}{}", &keys.url_piece, &query).parse()?;
    let mut buffer = BytesMut::with_capacity(1024);
    client.fetch_bytes(request, &mut buffer).await?;

    let (school_list_json, _, _) = encoding_rs::UTF_8.decode(&buffer[..]);   // Gets the school list
    let json_string = validate_json(&school_list_json);
    log::info!("{:?}", &buffer[..]);
    let school_list = serde_json::from_str::<SchoolList>(json_string.as_str()).unwrap().학교검색;

    Ok(school_list)
}

/// Removes invalid characters from the json string
pub fn validate_json(str: &str) -> String {
    str.chars().filter(|c| { c != &'\u{0}' }).collect::<String>()
}

/// Gets the keys for parsing the `RawSchoolData` struct from obfuscated JSON.
pub async fn init(client: &dyn ComciganClient) -> Result<RawSchoolDataKey> {
    let request = "http://comci.kr:4082/st".to_string();
    
    let mut buffer = BytesMut::with_capacity(1024);
    client.fetch_bytes(request, &mut buffer).await?;


    let (html, _, _) = encoding_rs::UTF_8.decode(&buffer[..]);
    let url_piece_regex = Regex::new(r#"(?<=\$\.ajax\({ url:'\.\/)(.*)(?='\+sc,success)"#).unwrap();

    let encode_header_regex = Regex::new(r#"(?<=sc_data\(')(.*)(?=',sc,1)"#).unwrap();

    let teachers_regex = Regex::new(r#"(?<=if\(th<자료\.)(.*)(?=\.length\))"#).unwrap();
    
    let timetable_regex = Regex::new(r#"(?<=일일자료=자료\.)(.*)(?=\[학년\]\[반\]\[요일\]\[교시\];if\(자료\.강의실==1)"#).unwrap();

    let subjects_regex = Regex::new(r#"(?<=속성\+"'>"\+자료\.)(.*)(?=\[sb\]\+"<br>"\+성명)"#).unwrap();

    let keys = RawSchoolDataKey {
        url_piece: url_piece_regex.find(&html)?.unwrap().as_str().to_string(),
        encode_header: encode_header_regex.find(&html)?.unwrap().as_str().to_string(),
        timetable: timetable_regex.find(&html)?.unwrap().as_str().to_string(),
        teachers: teachers_regex.find(&html)?.unwrap().as_str().to_string(),
        subjects: subjects_regex.find(&html)?.unwrap().as_str().to_string()
    };

    Ok(keys)
}

#[cfg(test)]
#[cfg(feature = "hyper")]
mod hyper_tests {
    use anyhow::Result;
    use crate::{init, search_school, view, client::HyperClient};

    #[tokio::test]
    async fn test() -> Result<()> {
        simple_logger::SimpleLogger::new().init().unwrap();
        use std::time::Instant;
        let now = Instant::now();
        let client = HyperClient::new();

        let keys = init(&client).await?;
        let schools = search_school(&client, "신목중", &keys).await?;
        let school = view(&client, &schools[0], &keys).await?;

        // 2학년 13반 금요일 4교시
        let day = school.grade(2).class(13).day(5);

        for period in day.list_periods() {
            println!("{}\n", period);
        }

        let then = Instant::now();

        println!("Time elapsed: {}", then.duration_since(now).as_millis());
        Ok(())
    }
}