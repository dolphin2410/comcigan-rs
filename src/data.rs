use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// List of 'School's searched with the given piece
pub struct SchoolList {
    pub(crate) 학교검색: Vec<School>
}

#[derive(Clone, Serialize, Deserialize)]
/// Representation of 'School'
pub struct School(pub u32, pub String, pub String, pub u32);

#[derive(Serialize, Deserialize)]
/// The raw json implementation of the school data
/// 
/// timetable: grade > class > day > period
/// 
/// subjects: list of subjects
/// 
/// teachers: list of teachers
pub struct RawSchoolData {
    pub timetable: Vec<Vec<Vec<Vec<u32>>>>,
    pub subjects: Vec<String>,
    pub teachers: Vec<String>
}

/// The keys for parsing the `RawSchoolData` struct from obfuscated JSON.
/// the values in this struct is in the following format: '자료000'
#[derive(Clone, Debug)]
pub struct RawSchoolDataKey {
    pub timetable: String,
    pub subjects: String,
    pub teachers: String,
    pub encode_header: String,
    pub url_piece: String
}