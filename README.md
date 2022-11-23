# 컴시간학생 API
## 컴시간학생 시간표 정보를 불러와줍니다

### Example Usage
```rust
use hyper::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new();
    let data = request_target(&client).await?;
    let schools = search_school(&client, "<여러분의 학교 이름>", &data).await?; // OO중 / OO고
    let school = view(&client, &schools[0], &data).await?;
    let period = school.grade(2).class(13).day(5).period(4); // 2학년 4반 금요일 4교시
    println!("Subject: {}", period.subject); // 과목 출력
    println!("Teacher: {}", period.teacher); // 교과 선생님 출력 
    Ok(())
}
```