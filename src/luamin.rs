use std::{fmt, rc::Rc};

use luamin::{
    luamin_bindgen,
    runtime::{CloneGasCost, PartialEqGasCost},
    smol_str::SmolStr,
    HostFuncLibArray, LuaminCompiler, LuaminExecutor, LuaminGas, LuaminUserValue, LuaminValue,
};
use rand::RngExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, LuaminUserValue)]
pub struct RustData(pub Rc<str>);

impl CloneGasCost for RustData {
    fn clone_gas_cost(&self) -> i32 {
        1
    }
}

impl PartialEqGasCost for RustData {
    fn partial_eq_gas_cost(&self, _other: &Self) -> i32 {
        self.0.len() as i32 + 1
    }
}

#[derive(Debug)]
struct HostError(String);

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HostError {}

fn incorrect_type(name: &'static str, index: usize) -> HostError {
    HostError(format!("Incorrect argument {index} to {name}"))
}

#[derive(Debug, LuaminExecutor)]
struct State;

#[luamin_bindgen(
    private_lifetime_provider_struct,
    incorrect_arg_type_error_func = incorrect_type,
    error_type = HostError,
    const_func_lib
)]
impl State {
    fn rand(&self, limit: i32) -> Result<i32, HostError> {
        if limit <= 0 {
            return Err(HostError("Random limit must be positive".into()));
        }
        Ok(rand::rng().random_range(0..limit))
    }

    fn rustdata_new(&self, codes: &[i32]) -> Result<RustData, HostError> {
        // Luamin has no native strings: assemble the script-generated ASCII
        // codes here. Other engines assemble strings before calling Rust.
        let mut text = String::with_capacity(codes.len());
        for &code in codes {
            let byte = u8::try_from(code)
                .ok()
                .filter(u8::is_ascii_hexdigit)
                .ok_or_else(|| HostError("Expected a hexadecimal character code".into()))?;
            text.push(char::from(byte));
        }
        Ok(RustData(text.into()))
    }

    fn rustdata_lt(&self, left: &RustData, right: &RustData) -> Result<bool, HostError> {
        Ok(left < right)
    }
}

pub fn sort_userdata(
    run: impl FnOnce(&mut dyn FnMut()),
    validate: impl FnOnce(Vec<LuaminValue>),
) -> anyhow::Result<()> {
    let lib = HostFuncLibArray::new(StateLifetimeProvider::new_func_def_statics());
    let mut compiler = LuaminCompiler::new(&lib, 1);
    let (script, _) = compiler.compile(
        &[SmolStr::new_static("count")],
        include_str!("../scripts/sort_userdata.luamin"),
    )?;
    let state = State;
    let execute = || {
        // MAX is a finite budget; reset it for every workload execution.
        let mut gas = LuaminGas::MAX;
        let mut values = state.execute_script(&mut gas, &script, &[LuaminValue::Number(10_000)])?;
        Ok::<_, anyhow::Error>(values.next().expect("Script must return its sorted array"))
    };

    let LuaminValue::List(result) = execute()? else {
        anyhow::bail!("Expected the script to return a list");
    };
    validate(
        Rc::try_unwrap(result)
            .expect("Result must not be aliased")
            .into_inner(),
    );
    run(&mut || {
        execute().unwrap();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use luamin::{IntoLuamin, TryFromLuamin};

    #[test]
    fn quicksort_preserves_values_and_handles_edge_cases() {
        let lib = HostFuncLibArray::new(StateLifetimeProvider::new_func_def_statics());
        let mut compiler = LuaminCompiler::new(&lib, 1);
        // Run the actual sorting section with known input instead of generating
        // random data, so we can verify permutation preservation as well as order.
        let (_, source) = include_str!("../scripts/sort_userdata.luamin")
            .split_once("-- BEGIN SORT")
            .unwrap();
        let (script, _) = compiler
            .compile(
                &[SmolStr::new_static("input")],
                &format!("local array = input\n{source}"),
            )
            .unwrap();
        let cases: &[&[&str]] = &[
            &[],
            &["a"],
            &["f", "0"],
            &["0", "1", "a", "f"],
            &["f", "a", "1", "0"],
            &["a", "a", "a", "a"],
            &["a", "0", "a", "f", "0", "aa", "a0", "a"],
        ];
        for &input in cases {
            let array = input
                .iter()
                .map(|&s| RustData(s.into()).into_luamin())
                .collect::<Vec<_>>()
                .into_luamin();
            let mut gas = LuaminGas::MAX;
            let mut returned = State.execute_script(&mut gas, &script, &[array]).unwrap();
            let LuaminValue::List(result) = returned.next().unwrap() else {
                panic!("Expected sorted list");
            };
            let actual = result
                .borrow()
                .iter()
                .cloned()
                .map(|value| RustData::try_from_luamin(value).unwrap().0.to_string())
                .collect::<Vec<_>>();
            let mut expected = input.to_vec();
            expected.sort();
            assert_eq!(actual, expected, "input: {input:?}");
        }
    }

    #[test]
    fn host_bindings_construct_and_compare_strings() {
        let state = State;
        let left = state.rustdata_new(&[48, 97]).unwrap();
        let right = state.rustdata_new(&[48, 98]).unwrap();
        assert_eq!(&*left.0, "0a");
        assert!(state.rustdata_lt(&left, &right).unwrap());
        assert!(!state.rustdata_lt(&left, &left).unwrap());
        assert!(state.rustdata_new(&[-1]).is_err());
        assert!(state.rustdata_new(&[256]).is_err());
        assert!(state.rustdata_new(&[103]).is_err());
    }
}
