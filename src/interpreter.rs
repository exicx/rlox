enum ExprResult {
    Nil,
    Bool(bool),
    Number(f64),
    LoxString(String),
}

impl ExprResult {
    fn interpret(&mut self) {}
}

// eval literal
// eval grouping -> expression
// eval unary minus
// eval unary not
// truthiness()
// eval binary operators (+, -, *, /)
// eval string concatenation (binary +)
// eval comparisons (>, >=, <, <=)
// eval equality (==, !=)
// equality()
// errors
