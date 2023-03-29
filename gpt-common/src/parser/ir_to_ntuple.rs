use crate::dto::{BoolDTO, BoolExpression, Input, IntervalDTO, NTupleInput};

use super::ir::{self, Feature};

const fn convert_bool_dto(condition: &ir::BoolCondition) -> BoolDTO {
    let expression = if condition.should_equal_to == true {
        BoolExpression::IsTrue
    } else {
        BoolExpression::IsFalse
    };
    BoolDTO {
        expression,
        bool_val: condition.should_equal_to,
        is_constant: false,
    }
}

fn convert_interval_dto(variable: &ir::Variable, condition: &ir::IntervalCondition) -> IntervalDTO {
    let precision = variable.var_type.get_precision().expect("Type error: when converting an interval dto in convert_interval_dto, the variable type doesn't have a precision!");

    IntervalDTO {
        expression: condition.expression,
        interval: condition.interval.clone(),
        precision,
        is_constant: false,
    }
}

fn convert_condition(variable: &ir::Variable, condition: ir::Condition) -> Input {
    match condition {
        ir::Condition::Bool(cond) => Input::Bool(convert_bool_dto(&cond)),
        ir::Condition::Interval(cond) => Input::Interval(convert_interval_dto(variable, &cond)),
    }
}

// fn sort_objects_into_tuple<TTupleElem, TObject, TCompare>(
//     tuple_format: Vec<TTupleElem>,
//     objects: &Vec<TObject>,
//     lens_tuple_elem: &mut dyn FnMut(&TTupleElem) -> TCompare,
//     lens_object: &mut dyn FnMut(&TObject) -> TCompare,
// ) -> Vec<(TTupleElem, Option<TObject>)>
// where
//     TObject: Copy,
//     TCompare: PartialEq,
// {
//     let objs = objects.clone();

//     tuple_format
//         .into_iter()
//         .map(|tuple_elem| {
//             let foo: Option<TObject> = objs
//                 .clone()
//                 .into_iter()
//                 .find(|x| lens_object(x) == lens_tuple_elem(&tuple_elem));

//             (tuple_elem, foo)
//         })
//         .collect()
// }

fn convert_predicate_to_ntuple(
    variables: &[ir::Variable],
    predicate: &ir::Predicate,
) -> NTupleInput {
    let inputs = predicate
        .clone()
        .into_iter()
        .map(|condition| {
            let variable = variables
                .iter()
                .find(|variable| condition.get_variable() == variable.var_name)
                // TODO: This should be an actual error in a Result type
                .unwrap_or_else(|| panic!("Undefined variable: {}", condition.get_variable()));
            (
                variable.var_name.to_owned(),
                convert_condition(variable, condition),
            )
        })
        .collect();

    NTupleInput { inputs }
}

pub fn ir_to_ntuple(
    Feature {
        variables,
        predicates,
    }: &Feature,
) -> Vec<NTupleInput> {
    predicates
        .clone()
        .iter()
        .map(|predicate| convert_predicate_to_ntuple(variables, predicate))
        .collect()
}
