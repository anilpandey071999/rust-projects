type WaterFallResult = Result<String, Box<dyn std::error::Error>>;

pub async fn task1() -> WaterFallResult {
    Ok("Task 1 completed".to_string())
}

pub async fn task2(input: String) -> WaterFallResult {
    Ok(format!("{} Task 2 completed", input))
}

pub async fn task3(input: String) -> WaterFallResult {
    Ok(format!("{} Task 3 completed", input))
}

#[tokio::test]
async fn test_main() -> Result<(), Box<dyn std::error::Error>> {
    let output1 = task1().await?;
    let output2 = task2(output1).await?;
    let result = task3(output2).await?;
    println!("{}", result);
    Ok(())
}
