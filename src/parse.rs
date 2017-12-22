
enum Function {
    unary: fn<T>(& T) -> T,
    binary_same: fn<T>(& T, & T) -> T,
    binary_diff: fn<T, U, V>(& T, & U) -> V,
}

enum Type {
    number: f64,
    function: Function
}

struct Interpreter {
    input: String,
    output: std::any::Any;
}