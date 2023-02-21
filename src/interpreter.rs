// rlox: Lox interpreter/compiler in Rust.
//    Copyright 2023 James Smith <j@mes.sh>
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

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
