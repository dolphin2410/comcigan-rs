use std::collections::HashMap;
use bytes::BytesMut;
use class::{Grade, Class, Day, SchoolData};
use client::ComciganClient;
use fancy_regex::Regex;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use anyhow::Result;

use crate::util::pop_first_element;

pub mod class;
pub mod client;
pub mod util;

#[derive(Serialize, Deserialize)]
/// The raw json implementation of the school list
pub struct SchoolList {
    pub(crate) 학교검색: Vec<School>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct School(pub u32, pub String, pub String, pub u32);

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
    let request = format!("http://comci.net:4082/{}?{}", &target, &encoded).parse()?;

    let mut school_list = String::new();
    client.fetch_string(request, &mut school_list).await?;

    let json = validate_json(&school_list);
    
    // println!("JSON: {:?}", json);

    // Can't do this in structs because the keys aren't static
    let raw_data = serde_json::from_str::<HashMap<&str, Value>>(json.as_str()).unwrap();
 
    let teachers = serde_json::value::from_value::<Vec<String>>(raw_data.get(keys.teachers.as_str()).unwrap().to_owned()).unwrap(); // Get teachers list
    let subjects_value: Value = serde_json::value::from_value(raw_data.get(keys.subjects.as_str()).unwrap().to_owned()).unwrap(); // Get subjects list
    
    let mut subjects_vec = subjects_value.as_array().unwrap().clone();
    subjects_vec.remove(0);
    let subjects = subjects_vec.iter().map(|x| {
        x.to_string()
    }).collect::<Vec<String>>();
    
    let timetable_value = serde_json::value::from_value::<Value>(raw_data.get(keys.timetable.as_str()).unwrap().to_owned()).unwrap();   // Get timetable list
    let timetable_vec = pop_first_element(&timetable_value);
    // println!("{:?}", timetable_vec);
    let timetable: Vec<Vec<Vec<Vec<u32>>>> = serde_json::from_value(timetable_vec).unwrap();
    
    let data = RawSchoolData {
        teachers,
        subjects,
        timetable
    };

    let mut school_data = SchoolData { name: school.2.clone(), grades: vec![] };  // todo fix school name
    for grade_index in 0..data.timetable.len() {
        let mut grade = Grade::new(grade_index as u8);

        for class_index in 0..data.timetable[grade_index].len() {
            let mut class = Class::new(class_index as u8);

            for days_index in 0..data.timetable[grade_index][class_index].len() {
                let mut day = Day::new(days_index as u8);

                for index in 0..data.timetable[grade_index][class_index][days_index].len() {
                    let subj_data = data.timetable[grade_index][class_index][days_index][index];
                    let th = (subj_data as f32 / 1000.0).floor() as u32;
                    let code = subj_data - (th * 1000);
                    let mut subject = data.subjects[th as usize - 1].clone();
                    let mut teacher = data.teachers[(code % 1000) as usize].clone();
                    if subject == "19" { // 28 32 25
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
    let request = format!("http://comci.net:4082/{}{}", &keys.url_piece, &query);
    let mut school_list_json = String::new();
    client.fetch_string(request, &mut school_list_json).await?;

    let json_string = validate_json(&school_list_json);
    let school_list = serde_json::from_str::<SchoolList>(json_string.as_str()).unwrap().학교검색;

    Ok(school_list)
}

/// Removes invalid characters from the json string
pub fn validate_json(str: &str) -> String {
    str.chars().filter(|c| { c != &'\u{0}' }).collect::<String>()
}

/// Gets the keys for parsing the `RawSchoolData` struct from obfuscated JSON.
pub async fn init(client: &dyn ComciganClient) -> Result<RawSchoolDataKey> {
    let request = "http://comci.net:4082/st".to_string();
    
    let mut buffer = BytesMut::with_capacity(1024);
    client.fetch_bytes(request, &mut buffer).await?;

    let (html, _, _) = encoding_rs::EUC_KR.decode(&buffer[..]);

    // println!("HTML: {}", html);

    let url_piece_regex = Regex::new(r#"(?<=\$\.ajax\({ url:'\.\/)(.*)(?='\+sc,success)"#).unwrap();

    let encode_header_regex = Regex::new(r#"(?<=sc_data\(')(.*)(?=',sc,1)"#).unwrap();

    let teachers_regex = Regex::new(r#"(?<=분리;if\(th<자료\.)(자료[1-9]*)(?=\.length\) {성명)"#).unwrap();
    
    let timetable_regex = Regex::new(r#"(?<=원자료=Q자료\(자료\.)(자료[0-9]*)(?=\[학년\]\[반\]\[요일\]\[교시\])"#).unwrap();

    let subjects_regex = Regex::new(r#"(?<=2px;'>"\+m2\+자료\.)(.*)(?=\[sb\]\+"<br>"\+성명)"#).unwrap();

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
mod hyper_tests {
    use anyhow::Result;
    use crate::{init, search_school, view};

    #[tokio::test]
    #[cfg(feature = "hyper")]
    async fn test() -> Result<()> {
        use crate::client::HyperClient;

        simple_logger::SimpleLogger::new().init().unwrap();
        use std::time::Instant;
        let now = Instant::now();
        let client = HyperClient::new();

        let keys = init(&client).await?;
        let schools = search_school(&client, "세종과학고등학교", &keys).await?;
        let school = view(&client, &schools[0], &keys).await?;

        // 1학년 1반 금요일 4교시
        let day = school.grade(1).class(1).day(5);

        for period in day.list_periods() {
            println!("{}\n", period);
        }

        log::warn!("WA SANS");

        let then = Instant::now();

        println!("Time elapsed: {}", then.duration_since(now).as_millis());
        Ok(())
    }
}
