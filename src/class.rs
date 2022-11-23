use std::{fmt::Display};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Grade {
    pub grade_num: u8,
    pub(crate) classes: Vec<Class>
}

impl Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} 학년", self.grade_num)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Class {
    pub class_num: u8,
    pub(crate) days: Vec<Day>
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} 반", self.class_num)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Day {
    pub(crate) day_num: u8,
    pub periods: Vec<Period>
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let day_num = self.day_num % 7;
        match day_num {
            0 => write!(f, "월요일"),
            1 => write!(f, "화요일"),
            2 => write!(f, "수요일"),
            3 => write!(f, "목요일"),
            4 => write!(f, "금요일"),
            5 => write!(f, "토요일"),
            6 => write!(f, "일요일"),
            _ => panic!("Invalid day number: {}", day_num)
        }
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Period {
    pub period_num: u8,
    pub subject: String,
    pub teacher: String
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}교시\n선생님: {}\n과목: {}", self.period_num, self.teacher, self.subject)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SchoolData {
    pub name: String, 
    pub(crate) grades: Vec<Grade>
}

impl Display for SchoolData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl SchoolData {
    pub fn grade(&self, grade: usize) -> Grade {
        self.grades[grade - 1].clone()
    }

    pub fn list_grades(&self) -> Vec<Grade> {
        self.grades.clone()
    }
}

impl Grade {
    pub fn class(&self, class: usize) -> Class {
        self.classes[class - 1].clone()
    }

    pub fn list_classes(&self) -> Vec<Class> {
        self.classes.clone()
    }

    pub fn new(grade_num: u8) -> Grade {
        Grade {
            grade_num,
            classes: Vec::new()
        }
    }
}

impl Class {
    pub fn day(&self, day: usize) -> Day {
        self.days[day - 1].clone()
    }

    pub fn list_days(&self) -> Vec<Day> {
        self.days.clone()
    }

    pub fn new(class_num: u8) -> Class {
        Class {
            class_num,
            days: Vec::new()
        }
    }
}

impl Day {
    pub fn period(&self, period: usize) -> Period {
        self.periods[period - 1].clone()
    }

    pub fn list_periods(&self) -> Vec<Period> {
        self.periods.clone()
    }

    pub fn new(day_num: u8) -> Day {
        Day {
            day_num,
            periods: Vec::new()
        }
    }
}