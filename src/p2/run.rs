use super::binding::Binding;
use super::error::{EBADF, Error};
use super::limits::Limits;
use super::state::{Callback, HostState};
use super::validate;
use wasmi::{Caller, Config, Engine, Linker, Module, Store};

pub(crate) fn run<'i, 'c>(
    wasm: &[u8],
    input: &'i [u8],
    cb: Option<Callback<'c>>,
    binding: &Binding,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, wasm).map_err(|_| Error::InvalidWasm)?;
    validate::check(&module, binding)?;
    let mut store = Store::new(&engine, HostState::new(input, cb, limits));
    store.set_fuel(limits.fuel()).map_err(|_| Error::Budget)?;
    store.limiter(|state| &mut state.resources);
    let mut linker = Linker::new(&engine);
    let wanted = binding.context();
    linker
        .func_wrap(
            binding.module(),
            binding.function(),
            move |mut caller: Caller<'_, HostState<'_, '_>>,
                  ctx: i32,
                  op: i32,
                  value: i32|
                  -> i64 {
                if ctx != wanted {
                    return i64::from(EBADF);
                }
                super::invoke::dispatch(caller.data_mut(), op, value)
            },
        )
        .map_err(|_| Error::Link)?;
    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .map_err(trap)?;
    instance
        .get_typed_func::<(), ()>(&store, "_start")
        .map_err(|_| Error::MissingStart)?
        .call(&mut store, ())
        .map_err(trap)?;
    store.into_data().finish().map_err(Error::Callback)
}

fn trap(e: wasmi::Error) -> Error {
    match e.as_trap_code() {
        Some(wasmi::TrapCode::OutOfFuel) => Error::Budget,
        _ => Error::Trap,
    }
}
