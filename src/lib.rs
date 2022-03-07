use class::{Grade, Class, Day, SchoolData};
use hyper::{body::HttpBody as _, Client, client::HttpConnector};
use serde::{Serialize, Deserialize};

pub mod class;

#[derive(Serialize, Deserialize)]
pub struct SchoolList {
    pub 학교검색: Vec<School>
}

#[derive(Serialize, Deserialize)]
pub struct School(u32, String, String, u32);

#[derive(Serialize, Deserialize)]
pub struct RawSchoolData {
    pub 자료311: Vec<Vec<Vec<Vec<u32>>>>,
    pub 자료565: Vec<String>,
    pub 자료389: Vec<String>
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
    let raw_id = format!("106686_{}_0_1", school.3);
    let encoded = base64::encode(raw_id);

    let request = format!("http://comci.kr:4082/163398?{}", &encoded).parse()?;
    let mut response = client.get(request).await?;

    let mut buffer = vec![];
    while let Some(chunk) = response.body_mut().data().await {
        buffer.append(&mut chunk?.to_vec());
    }

    let (school_list, _, _) = encoding_rs::UTF_8.decode(buffer.as_slice());
    let data = serde_json::from_str::<RawSchoolData>(validate_json(&school_list).as_str()).unwrap();

    buffer.clear();

    let mut to_return = SchoolData { grades: vec![] };
    for grade_index in 1..data.자료311.len() {
        let mut grade = Grade { classes: vec![] };
        for class_index in 1..data.자료311[grade_index].len() {
            let mut class = Class { days: vec![] };
            for days_index in 1..data.자료311[grade_index][class_index].len() {
                let mut day = Day { studies: vec![] };
                for index in 1..data.자료311[grade_index][class_index][days_index].len() {
                    let subj_data = data.자료311[grade_index][class_index][days_index][index];
                    let th = (subj_data as f32 / 100.0).floor() as u32;
                    let code = subj_data - (th * 100);
                    let mut subject = data.자료565[code as usize].clone();
                    let mut teacher = data.자료389[th as usize].clone();
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

    let request = format!("http://comci.kr:4082/163398?72294l{}", &query).parse()?;
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