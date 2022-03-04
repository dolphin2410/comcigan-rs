# 컴시간학생 API
## 컴시간학생 시간표 정보를 불러와줍니다

### Example Usage
```rust
use hyper::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let schools = search_school(&client, "<여러분의 학교 이름>").await?; // OO중 / OO고
    let school = view(&client, &schools[0]).await?;
    let study = school.grade(2).class(4).day(5).study(4); // 2학년 4반 금요일 4교시
    println!("Subject: {}", study.subject); // 과목 출력
    println!("Teacher: {}", study.teacher); // 교과 선생님 출력 
    Ok(())
}
```

### Dependencies
Hyper-rs -> HTTP 클라이언트 역할을 합니다
Encoding-rs -> 옛 jQuery의 EUC_KR 인코딩용
Serde-rs -> JSON 파싱용
Base64 -> Base64 인코딩용
Tokio-rs -> Hyper-rs 번들