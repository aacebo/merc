fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rust_bert::pipelines::question_answering::{QaInput, QuestionAnsweringModel};

    let qa_model = QuestionAnsweringModel::new(Default::default())?;
    let question = String::from("Where does Amy live ?");
    let context = String::from("Amy lives in Amsterdam");
    let answers = qa_model.predict(&[QaInput { question, context }], 1, 32);

    for arr in &answers {
        for (i, answer) in arr.iter().enumerate() {
            println!("Answer #{}: {} = {}", i, answer.answer, answer.score);
        }
    }

    Ok(())
}
