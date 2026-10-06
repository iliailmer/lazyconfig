use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    Number(f64),
    Text(String),
    Bool(bool),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(number) if number.fract() == 0.0 => write!(formatter, "{number:.1}"),
            Self::Number(number) => write!(formatter, "{number}"),
            Self::Text(text) => formatter.write_str(text),
            Self::Bool(flag) => write!(formatter, "{flag}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Fixed,
    Number { by: f64, min: f64, max: f64 },
    Toggle,
    Choices(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditableField {
    pub id: String,
    pub label: String,
    pub value: FieldValue,
    pub step: Step,
}

impl EditableField {
    pub fn new(id: &str, label: &str, value: FieldValue) -> Self {
        let step = match value {
            FieldValue::Number(_) => Step::Number {
                by: 1.0,
                min: f64::MIN,
                max: f64::MAX,
            },
            FieldValue::Bool(_) => Step::Toggle,
            FieldValue::Text(_) => Step::Fixed,
        };

        Self {
            id: id.into(),
            label: label.into(),
            value,
            step,
        }
    }

    pub fn from_number(id: &str, label: &str, value: f64) -> Self {
        Self::new(id, label, FieldValue::Number(value))
    }

    pub fn from_string(id: &str, label: &str, value: &str) -> Self {
        Self::new(id, label, FieldValue::Text(value.into()))
    }

    pub fn from_bool(id: &str, label: &str, value: bool) -> Self {
        Self::new(id, label, FieldValue::Bool(value))
    }

    pub fn with_range(mut self, by: f64, min: f64, max: f64) -> Self {
        self.step = Step::Number { by, min, max };
        self
    }

    pub fn with_choices(mut self, choices: Vec<String>) -> Self {
        self.step = Step::Choices(choices);
        self
    }

    pub fn stepped(&self, direction: Direction) -> Option<FieldValue> {
        let next = match (&self.value, &self.step) {
            (FieldValue::Number(number), Step::Number { by, min, max }) => {
                let delta = match direction {
                    Direction::Up => *by,
                    Direction::Down => -*by,
                };
                let moved = ((number + delta) * 100.0).round() / 100.0;
                FieldValue::Number(moved.clamp(*min, *max))
            }
            (FieldValue::Bool(flag), Step::Toggle) => FieldValue::Bool(!flag),
            (FieldValue::Text(text), Step::Choices(choices)) => {
                let index = choices.iter().position(|choice| choice == text)?;
                let count = choices.len();
                let next_index = match direction {
                    Direction::Up => (index + 1) % count,
                    Direction::Down => (index + count - 1) % count,
                };
                FieldValue::Text(choices[next_index].clone())
            }
            _ => return None,
        };

        (next != self.value).then_some(next)
    }
}
