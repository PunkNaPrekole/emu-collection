#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Pass,
    /// присваивание, например: v0 = 10
    Assign {
        target: String,
        value: Expression,
    },
    /// Рисовать символ, пример: print(x, y, "A")
    Print {
        x: Expression,
        y: Expression,
        character: char,
    },
    /// if условие: ... 
    If {
        condition: Condition,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    /// while условие: ...
    While {
        condition: Condition,
        body: Vec<Statement>,
    },
    /// jump("label")
    Jump {
        label: String,
    },
    /// label:
    Label {
        name: String,
    },
    /// Очистка экрана, пример: clear()
    ClearScreen,
    /// Задержка в секундах, пример: sleep(5)
    Delay {
        frames: Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    /// 10, 0xFF, 0b1010 (decimal, hex, binary)
    Number(u16),
    /// v0, x, y
    Variable(String),
    /// v0 + 5
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    True,
    /// v0 == 5
    Equal(Expression, Expression),
    /// v0 != 5  
    NotEqual(Expression, Expression),
    /// v0 > 5
    Greater(Expression, Expression),
    /// key_pressed(1)
    KeyPressed(Expression),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Or,       // |
    And,      // &
    Xor,      // ^
}