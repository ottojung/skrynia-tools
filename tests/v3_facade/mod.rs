mod app_flow;
mod check_cas;
mod check_list;
mod fake;
mod http;
mod seed;

#[test]
fn explicit_facade_lists_all_apps_and_rolls_back_atomically() {
    app_flow::run();
}
