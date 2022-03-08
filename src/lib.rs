use std::collections::HashMap;

use class::{Grade, Class, Day, SchoolData};
use hyper::{body::HttpBody as _, Client, client::HttpConnector};
use fancy_regex::Regex;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub mod class;

#[derive(Serialize, Deserialize)]
pub struct SchoolList {
    pub 학교검색: Vec<School>
}

#[derive(Serialize, Deserialize)]
pub struct School(u32, String, String, u32);

#[derive(Serialize, Deserialize)]
pub struct RawSchoolData {
    pub timetable: Vec<Vec<Vec<Vec<u32>>>>,
    pub subjects: Vec<String>,
    pub teachers: Vec<String>
}

pub struct RawSchoolDataKey {
    pub timetable: String,
    pub subjects: String,
    pub teachers: String,
    pub encode_header: String,
    pub url_piece: String
}

#[tokio::test]
async fn test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let schools = search_school(&client, "향동중").await?;
    let school = view(&client, &schools[0]).await?;
    let study = school.grade(2).class(4).day(5).study(4);
    println!("Subject: {}", study.subject);
    println!("Teacher: {}", study.teacher);
    Ok(())
}

pub async fn view(client: &Client<HttpConnector>, school: &School) -> Result<SchoolData, Box<dyn std::error::Error + Send + Sync>> {
    let subtarget = request_target(client).await?;
    let raw_id = format!("{}{}_0_1", subtarget.encode_header, school.3);
    let encoded = base64::encode(raw_id);


    let target = subtarget.url_piece.split("?").nth(0).unwrap();
    let request = format!("http://comci.kr:4082/{}?{}", &target, &encoded).parse()?;
    let mut response = client.get(request).await?;

    let mut buffer = vec![];
    while let Some(chunk) = response.body_mut().data().await {
        buffer.append(&mut chunk?.to_vec());
    }

    let (school_list, _, _) = encoding_rs::UTF_8.decode(buffer.as_slice());
    let json = validate_json(&school_list);
    let raw_data = serde_json::from_str::<HashMap<&str, Value>>(json.as_str()).unwrap();
    let teachers = serde_json::value::from_value::<Vec<String>>(raw_data.get(subtarget.teachers.as_str()).unwrap().to_owned()).unwrap();
    let subjects = serde_json::value::from_value::<Vec<String>>(raw_data.get(subtarget.subjects.as_str()).unwrap().to_owned()).unwrap();
    let timetable = serde_json::value::from_value::<Vec<Vec<Vec<Vec<u32>>>>>(raw_data.get(subtarget.timetable.as_str()).unwrap().to_owned()).unwrap();
    
    let data = RawSchoolData {
        teachers,
        subjects,
        timetable
    };

    buffer.clear();

    let mut to_return = SchoolData { grades: vec![] };
    for grade_index in 1..data.timetable.len() {
        let mut grade = Grade { classes: vec![] };
        for class_index in 1..data.timetable[grade_index].len() {
            let mut class = Class { days: vec![] };
            for days_index in 1..data.timetable[grade_index][class_index].len() {
                let mut day = Day { studies: vec![] };
                for index in 1..data.timetable[grade_index][class_index][days_index].len() {
                    let subj_data = data.timetable[grade_index][class_index][days_index][index];
                    let th = (subj_data as f32 / 100.0).floor() as u32;
                    let code = subj_data - (th * 100);
                    let mut subject = data.subjects[code as usize].clone();
                    let mut teacher = data.teachers[th as usize].clone();
                    if subject == "19" {
                        subject.clear();
                        subject.push_str("Nothing to study...");

                        teacher.clear();
                        teacher.push_str("No Teacher for this class...");
                    }
                    day.studies.push(class::Study { subject, teacher })
                }
                class.days.push(day)
            }
            grade.classes.push(class);
        }
        to_return.grades.push(grade);
    }

    Ok(to_return)
}

pub async fn search_school(client: &Client<HttpConnector>, school: &'static str) -> Result<Vec<School>, Box<dyn std::error::Error + Send + Sync>> {
    let (result, _, _) = encoding_rs::EUC_KR.encode(school);
    let query: String = result.iter().map(|byte| format!("%{:X}", byte)).collect();

    let target = request_target(client).await?;

    let request = format!("http://comci.kr:4082/{}{}", &target.url_piece, &query).parse()?;
    let mut response = client.get(request).await?;
    
    let mut buffer = vec![];
    while let Some(chunk) = response.body_mut().data().await {
        buffer.append(&mut chunk?.to_vec());
    }

    let (school_list, _, _) = encoding_rs::UTF_8.decode(buffer.as_slice());

    Ok(serde_json::from_str::<SchoolList>(validate_json(&school_list).as_str()).unwrap().학교검색)
}

pub fn validate_json(str: &str) -> String {
    str.chars().filter(|c| { c != &'\u{0}' }).collect::<String>()
}

pub async fn request_target(client: &Client<HttpConnector>) -> Result<RawSchoolDataKey, Box<dyn std::error::Error + Send + Sync>> {
    let request = "http://comci.kr:4082/st".parse()?;
    let mut response = client.get(request).await?;
    
    let mut buffer = vec![];
    while let Some(chunk) = response.body_mut().data().await {
        buffer.append(&mut chunk?.to_vec());
    }

    let (html, _, _) = encoding_rs::EUC_KR.decode(buffer.as_slice());
    let re = Regex::new(r#"(?<=\$\.ajax\({ url:'\.\/)(.*)(?='\+sc,success)"#).unwrap();

    let re2 = Regex::new(r#"(?<=sc_data\(')(.*)(?=',sc,1)"#).unwrap();

    let re3 = Regex::new(r#"(?<=if\(th<자료\.)(.*)(?=\.length\))"#).unwrap();
    
    let re4 = Regex::new(r#"(?<=일일자료=자료\.)(.*)(?=\[학년\]\[반\]\[요일\]\[교시\];if\(자료\.강의실==1)"#).unwrap();

    let re5 = Regex::new(r#"(?<=속성\+"'>"\+자료\.)(.*)(?=\[sb\]\+"<br>"\+성명)"#).unwrap();

    let keys = RawSchoolDataKey {
        subjects: String::from(re5.find(&html).unwrap().unwrap().as_str()),
        teachers: String::from(re3.find(&html).unwrap().unwrap().as_str()),
        timetable: String::from(re4.find(&html).unwrap().unwrap().as_str()),
        encode_header: String::from(re2.find(&html).unwrap().unwrap().as_str()),
        url_piece: String::from(re.find(&html).unwrap().unwrap().as_str())
    };

    Ok(keys)
}