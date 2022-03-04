pub struct Grade {
    pub classes: Vec<Class>
}

pub struct Class {
    pub days: Vec<Day>
}

pub struct Day {
    pub studies: Vec<Study>
}

pub struct Study {
    pub subject: String,
    pub teacher: String
}

pub struct SchoolData {
    pub grades: Vec<Grade>
}

impl SchoolData {
    pub fn grade(&self, grade: usize) -> &Grade {
        &self.grades[grade - 1]
    }
}

impl Grade {
    pub fn class(&self, class: usize) -> &Class {
        &self.classes[class - 1]
    }
}

impl Class {
    pub fn day(&self, day: usize) -> &Day {
        &self.days[day - 1]
    }
}

impl Day {
    pub fn study(&self, study: usize) -> &Study {
        &self.studies[study - 1]
    }
}